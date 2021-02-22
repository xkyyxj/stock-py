use chrono::prelude::*;
use std::ops::Add;
use async_std::task::sleep;
use chrono::Duration;
use crate::results::{TimeIndexInfo, TimeIndexBaseInfo, TimeIndexBatchInfo, DBResult};
use std::str::FromStr;
use crate::cache::AsyncRedisOperation;


pub(crate) static INDEX_SUFFIX: &str = "_index";

/// 获取股票的实时信息
pub async fn fetch_index_info(stock_code: Vec<String>) {
    // 知道为什么println!不生效么？可能因为多个线程同时竞争了，导致打印有问题！！！
    // println!("6666666666666666！！！！！！！");
    let mut redis_ope = AsyncRedisOperation::new().await;
    // 构建查询的URL地址
    let mut target_string = String::from("http://hq.sinajs.cn/list=");
    for item in &stock_code {
        if item.contains("SZ") {
            // 深交所的
            target_string = target_string.add("sz");
        }
        else {
            // 上交所的
            target_string = target_string.add("sh");
        }
        target_string = target_string.add(item.get(..6).unwrap());
        target_string = target_string.add(",");
    }

    let local: DateTime<Local> = Local::now();
    let year = local.date().year();
    let month = local.date().month();
    let day = local.date().day();
    // 当前天上午开盘时间(上午9:29:59)
    let mut up_begin_time = Local.ymd(year, month, day).and_hms_milli(9, 29, 59, 0);
    // 当前天上午闭盘时间(上午11:59:59)
    let mut up_end_time = Local.ymd(year, month, day).and_hms_milli(11, 29, 59, 0);
    // 当前天下午开盘时间(上午12:59:59)
    let mut down_begin_time = Local.ymd(year, month, day).and_hms_milli(12, 59, 59, 0);
    // 当前天下午闭盘时间(上午14:59:59)
    let mut down_end_time = Local.ymd(year, month, day).and_hms_milli(14, 59, 59, 0);

    loop {
        // 当前时间在开盘时间之内
        let curr_time = Local::now();
        if (curr_time >= up_begin_time && curr_time <= up_end_time) ||
            (curr_time >= down_begin_time && curr_time <= down_end_time) {
            let ret_rst = surf::get(target_string.as_str()).await;
            if let Err(_) = ret_rst {
                continue;
            }
            let mut res = ret_rst.unwrap();
            let ret_val_rst = res.body_string().await;
            if let Err(_) = ret_val_rst {
                continue;
            }
            let ret_val = ret_val_rst.unwrap();
            split_multi_info(ret_val, &mut redis_ope).await;

            // 每两秒获取一次
            let two_seconds_duration = Duration::seconds(crate::config::INDEX_INFO_FETCH_DELTA);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = two_seconds_duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
        }

        // 早上未开盘之前，休眠
        if curr_time < up_begin_time {
            let temp_duration = (up_begin_time - curr_time).to_std().unwrap();
            del_cache(&stock_code, &mut redis_ope).await;
            sleep(temp_duration).await;
        }

        // 上午到下午之间的间歇，休眠
        if curr_time > up_end_time && curr_time <= down_begin_time {
            let temp_duration = (down_begin_time - curr_time).to_std().unwrap();
            // TODO -- 内存不足，redis hold不住了，先这样处理吧；另外可以考虑压缩，后者压缩后存储到磁盘上去
            // del_cache(&stock_code, &mut redis_ope).await;
            sleep(temp_duration).await;
            println!("sleep finished {}", curr_time);
        }

        // 到了第二天，呵呵哒哒
        if curr_time > down_end_time {
            // 午夜三点左右，删除前一天的redis缓存吧
            let del_redis_duration = Duration::hours(12);
            let next_three_morning = down_end_time + del_redis_duration;
            let mut temp_duration = (next_three_morning - curr_time).to_std().unwrap();
            sleep(temp_duration).await;
            del_cache(&stock_code, &mut redis_ope).await;
            println!("delete redis cache time is {}", next_three_morning);

            let after_del_time = Local::now();
            let next_day_duration = Duration::hours(24);
            up_begin_time = up_begin_time.add(next_day_duration);
            temp_duration = (up_begin_time - after_del_time).to_std().unwrap();
            up_end_time = up_end_time.add(next_day_duration);
            down_begin_time = down_begin_time.add(next_day_duration);
            down_end_time = down_end_time.add(next_day_duration);
            println!("delete redis cache finished time is {}", after_del_time);
            sleep(temp_duration).await;
            println!("next new day time is {}", after_del_time);
        }
    }
}

async fn del_cache(ts_codes: &Vec<String>, redis_ope: &mut AsyncRedisOperation) {
    for item in ts_codes {
        println!("delete item {}", item);
        let redis_key = String::from(item) + INDEX_SUFFIX;
        redis_ope.delete(redis_key).await;
    }
}

async fn split_multi_info(content: String, redis_ope: &mut AsyncRedisOperation) {
    let v: Vec<&str> = content.split(';').collect();
    for item in v {
        process_single_info(String::from(item), redis_ope).await;
    }
}

async fn process_single_info(content: String, redis_ope: &mut AsyncRedisOperation) {
    let mut single_info = TimeIndexInfo::new();
    let v: Vec<&str> = content.split('=').collect();
    if v.len() < 2 {
        return;
    }
    let head_part = String::from(v[0]);
    // 处理一下股票编码
    if head_part.len() > 8 {
        let _ts_code = head_part.get(head_part.len() - 8..head_part.len()).unwrap();
        let real_code = _ts_code.get(_ts_code.len() - 6.._ts_code.len()).unwrap();
        let mut final_code = String::from(real_code);
        if _ts_code[0..2] == "sz"[0..2] {
            final_code = final_code + ".SZ";
        }
        else if _ts_code[0..2] == "sh"[0..2] {
            final_code = final_code + ".SH";
        }
        single_info.ts_code = final_code;
    }

    let mut main_content = String::from(v[1]);
    main_content.remove(0); // 去除一个引号
    main_content.pop(); // 去除最后一个引号
    // 主体部分
    let main_content_v: Vec<&str> = main_content.split(',').collect();
    if main_content_v.len() > 32 {  // 普通股票
        single_info.ts_name = String::from(main_content_v[0]);
        single_info.t_open = String::from(main_content_v[1]).parse().unwrap();
        single_info.y_close = String::from(main_content_v[2]).parse().unwrap();
        single_info.curr_price = String::from(main_content_v[3]).parse().unwrap();
        single_info.t_max = String::from(main_content_v[4]).parse().unwrap();
        single_info.t_min = String::from(main_content_v[5]).parse().unwrap();
        single_info.deal_num = String::from(main_content_v[8]).parse().unwrap();
        single_info.deal_mny = String::from(main_content_v[9]).parse().unwrap();
        for i in 0..5 {
            single_info.buy_num[i] = String::from(main_content_v[10 + i * 2]).parse().unwrap();
            single_info.buy_price[i] = String::from(main_content_v[10 + i * 2 + 1]).parse().unwrap();
        }
        for i in 0..5 {
            single_info.sold_num[i] = String::from(main_content_v[20 + i * 2]).parse().unwrap();
            single_info.sold_price[i] = String::from(main_content_v[20 + i * 2 + 1]).parse().unwrap();
        }

        let date_time_str = String::from(main_content_v[30]) + "T" + main_content_v[31] + "+08:00";
        single_info.curr_time = DateTime::<Local>::from_str(date_time_str.as_str()).unwrap();
    }
    // 从redis当中获取到相关的数据，然后拼接，存储到redis当中
    let mut redis_key = String::from(&single_info.ts_code);
    redis_key = redis_key.add(INDEX_SUFFIX);

    if redis_ope.exists(redis_key).await {
        let write_str = single_info.to_string();
        // 增加一点小判定，耗费点CPU，节省点内存，反正rust的tokio够快
        redis_key = String::from(&single_info.ts_code);
        redis_key = redis_key.add(INDEX_SUFFIX);
        let length = redis_ope.str_length::<String>(redis_key).await;
        let mut start = length - 800;
        // FIXME -- 此处有一个redis模块(依赖的redis模块，而不是redis服务器)的BUG：如果get_range的start的index正好位于中文字符串的中间，就不能成功返回数据了，此处修正一下
        // FIXME -- 如果start_index小于150，直接到0，这样能够避免这个问题吧，毕竟中文只在开头有
        if start < 150 {
            start = 0;
        }
        redis_key = String::from(&single_info.ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let ret_str = redis_ope.get_range::<String, String>(redis_key, start, length).await.unwrap();
        // 处理一下字符串，获取到最新的实时信息
        let temp_infos: Vec<&str> = ret_str.split('~').collect();
        if temp_infos.len() > 2 {
            let last_info_str = String::from(*temp_infos.get(temp_infos.len() - 2).unwrap());
            let last_index_info: TimeIndexBaseInfo = last_info_str.into();
            // 如果两者相等的话，那么就不用在添加到缓存里面了
            if last_index_info == single_info.get_base_info() {
                return;
            }
        }

        redis_key = String::from(&single_info.ts_code);
        redis_key = redis_key.add(INDEX_SUFFIX);
        redis_ope.append_str(redis_key, write_str).await;
    }
    else {
        let mut index_batch_info = TimeIndexBatchInfo::new();
        index_batch_info.add_single_info(&single_info);
        let write_str = index_batch_info.to_string();
        redis_key = String::from(&single_info.ts_code);
        redis_key = redis_key.add(INDEX_SUFFIX);
        redis_ope.set::<String, String>(redis_key, write_str).await;
    }

    // 下面这种方案经验证，太过于耗费CPU，所有换种方式（就是上面那种方式）
    // match redis_ope.get::<String, String>(redis_key).await {
    //     Some(str) => {
    //         let mut output_str = String::from(&str);
    //         let mut index_batch_info = Box::new(TimeIndexBatchInfo::from(str));
    //         if let Some(last_info) = index_batch_info.get_last_info() {
    //             // 最后获取到的信息和上次获取到的信息一样的话，就不用存储了，节省点缓存
    //             if *last_info == single_info.get_base_info() {
    //                 return
    //             }
    //         }
    //         index_batch_info.add_single_info(&single_info);
    //         let mut write_str = index_batch_info.to_string();
    //         output_str = String::from(&write_str);
    //         redis_key = String::from(&single_info.ts_code);
    //         redis_key = redis_key.add(INDEX_SUFFIX);
    //         // redis_ope.set::<String, String>(redis_key, write_str).await;
    //     },
    //     None => {
    //         let mut index_batch_info = TimeIndexBatchInfo::new();
    //         index_batch_info.add_single_info(&single_info);
    //         let mut write_str = index_batch_info.to_string();
    //         redis_key = String::from(&single_info.ts_code);
    //         redis_key = redis_key.add(INDEX_SUFFIX);
    //         // redis_ope.set::<String, String>(redis_key, write_str).await;
    //     }
    // }
}