use hyper::Client;
use chrono::prelude::*;
use std::ops::Add;
use async_std::task::sleep;
use chrono::Duration;
use hyper::body::Buf;
use encoding::{DecoderTrap, Encoding};
use encoding::all::GBK;
use crate::results::{TimeIndexInfo, TimeIndexBatchInfo, DBResult};
use std::str::FromStr;
use crate::cache::AsyncRedisOperation;
use redis::aio::Connection;

pub(crate) static INDEX_SUFFIX: &str = "_index";

/// 获取股票的实时信息
pub async fn fetch_index_info(stock_code: Vec<String>) {
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
    // Still inside `async fn main`...
    let client = Client::new();

    let local: DateTime<Local> = Local::now();
    let year = local.date().year();
    let month = local.date().month();
    let day = local.date().day();
    // 当前天上午开盘时间(上午9:29:59)
    let mut _up_begin_time = Local.ymd(year, month, day).and_hms_milli(9, 29, 59, 0);
    // 当前天上午闭盘时间(上午11:59:59)
    let mut _up_end_time = Local.ymd(year, month, day).and_hms_milli(11, 59, 59, 0);
    // 当前天下午开盘时间(上午12:59:59)
    let mut _down_begin_time = Local.ymd(year, month, day).and_hms_milli(12, 59, 59, 0);
    // 当前天下午闭盘时间(上午14:59:59)
    let mut _down_end_time = Local.ymd(year, month, day).and_hms_milli(14, 59, 59, 0);

    loop {
        // 当前时间在开盘时间之内
        let curr_time = Local::now();
        if (curr_time >= _up_begin_time && curr_time <= _up_end_time) ||
            (curr_time >= _down_begin_time && curr_time <= _down_end_time) {
            // Parse an `http::Uri`...
            let uri = target_string.parse().unwrap();
            // Await the response...
            if let Ok(resp) = client.get(uri).await {
                if let Ok(content_rst) = hyper::body::to_bytes(resp.into_body()).await {
                    let ret_val = GBK.decode(content_rst.bytes(), DecoderTrap::Strict);
                    split_multi_info(ret_val.unwrap(), &mut redis_ope).await;

                    // 每两秒获取一次
                    let two_seconds_duration = Duration::seconds(crate::config::INDEX_INFO_FETCH_DELTA);
                    let fetch_finish_time = Local::now();
                    let fetch_cost_time = fetch_finish_time - curr_time;
                    let real_sleep_time = two_seconds_duration - fetch_cost_time;
                    if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                        sleep(real_sleep_time.to_std().unwrap()).await;
                    }
                }
            }
        }

        // 上午到下午之间的间歇，休眠
        if curr_time > _up_end_time && curr_time <= _down_begin_time {
            let temp_duration = (_down_begin_time - _up_end_time).to_std().unwrap();
            sleep(temp_duration).await;
        }

        // 到了第二天，呵呵哒哒
        if curr_time > _down_end_time {
            let next_day_duration = Duration::hours(24);
            _up_begin_time = _up_begin_time.add(next_day_duration);
            let temp_duration = (_up_begin_time - _down_end_time).to_std().unwrap();
            _up_end_time = _up_end_time.add(next_day_duration);
            _down_begin_time = _down_begin_time.add(next_day_duration);
            _down_end_time = _down_end_time.add(next_day_duration);
            del_cache(&stock_code, &mut redis_ope).await;
            sleep(temp_duration).await;
        }

    }
}

async fn del_cache(ts_codes: &Vec<String>, redis_ope: &mut AsyncRedisOperation) {
    for item in ts_codes {
        redis_ope.delete(item.as_str()).await;
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
    let mut head_part = String::from(v[0]);
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
    match redis_ope.get::<String, String>(redis_key).await {
        Some(str) => {
            let mut output_str = String::from(&str);
            let mut index_batch_info = Box::new(TimeIndexBatchInfo::from(str));
            if let Some(last_info) = index_batch_info.get_last_info() {
                // 最后获取到的信息和上次获取到的信息一样的话，就不用存储了，节省点缓存
                if *last_info == single_info.get_base_info() {
                    return
                }
            }
            index_batch_info.add_single_info(&single_info);
            let mut write_str = index_batch_info.to_string();
            output_str = String::from(&write_str);
            redis_key = String::from(&single_info.ts_code);
            redis_key = redis_key.add(INDEX_SUFFIX);
            redis_ope.set::<String, String>(redis_key, write_str).await;
        },
        None => {
            let mut index_batch_info = TimeIndexBatchInfo::new();
            index_batch_info.add_single_info(&single_info);
            let mut write_str = index_batch_info.to_string();
            redis_key = String::from(&single_info.ts_code);
            redis_key = redis_key.add(INDEX_SUFFIX);
            redis_ope.set::<String, String>(redis_key, write_str).await;
        }
    }
}