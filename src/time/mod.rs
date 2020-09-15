use hyper::Client;
use chrono::prelude::*;
use std::ops::Add;
use async_std::task::sleep;
use chrono::Duration;
use hyper::body::Buf;
use encoding::{DecoderTrap, Encoding};
use encoding::all::GBK;
use crate::results::{TimeIndexInfo, DBResult};
use std::str::FromStr;

static FETCH_DELTA_TIME: i32 = 10;

pub async fn fetch_index_info(stock_code: Vec<String>) {
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
    let _up_begin_time = Local.ymd(year, month, day).and_hms_milli(9, 29, 59, 0);
    // 当前天上午闭盘时间(上午11:59:59)
    let _up_end_time = Local.ymd(year, month, day).and_hms_milli(11, 59, 59, 0);
    // 当前天下午开盘时间(上午12:59:59)
    let _down_begin_time = Local.ymd(year, month, day).and_hms_milli(12, 59, 59, 0);
    // 当前天下午闭盘时间(上午14:59:59)
    let _down_end_time = Local.ymd(year, month, day).and_hms_milli(14, 59, 59, 0);

    println!("traget string is {}", target_string);
    loop {
        // 当前时间在开盘时间之内
        let curr_time = Local::now();
        // if (curr_time >= up_begin_time && curr_time <= up_end_time) ||
        //     (curr_time >= down_begin_time && curr_time <= down_end_time) {
            // Parse an `http::Uri`...
            let uri = target_string.parse().unwrap();
            // Await the response...
            match client.get(uri).await {
                Ok(resp) => {
                    let content_rst = hyper::body::to_bytes(resp.into_body()).await;
                    match content_rst {
                        Ok(val) => {
                            let ret_val = GBK.decode(val.bytes(), DecoderTrap::Strict);
                            split_multi_info(ret_val.unwrap());

                            // 每两秒获取一次
                            let two_seconds_duration = Duration::seconds(FETCH_DELTA_TIME as i64);
                            let fetch_finish_time = Local::now();
                            let fetch_cost_time = fetch_finish_time - curr_time;
                            let real_sleep_time = two_seconds_duration - fetch_cost_time;
                            println!("sleep time is {}", real_sleep_time.num_milliseconds());
                            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                                sleep(real_sleep_time.to_std().unwrap()).await;
                            }
                        },
                        Err(err) => {
                            //println!("content is {}",format!("{:?}", err));
                        }
                    }
                },
                Err(err) => {
                    //println!("err is {}",format!("{:?}", err));
                }
            };

        // }

        // 上午到下午之间的间歇，休眠
        // if curr_time > up_end_time && curr_time <= down_begin_time {
        //     let temp_duration = (down_begin_time - up_end_time).to_std().unwrap();
        //     sleep(temp_duration).await;
        // }
        //
        // // 到了第二天，呵呵哒哒
        // if curr_time > down_end_time {
        //     let next_day_duration = Duration::hours(24);
        //     up_begin_time = up_begin_time.add(next_day_duration);
        //     let temp_duration = (up_begin_time - down_end_time).to_std().unwrap();
        //     up_end_time = up_end_time.add(next_day_duration);
        //     down_begin_time = down_begin_time.add(next_day_duration);
        //     down_end_time = down_end_time.add(next_day_duration);
        //     sleep(temp_duration).await;
        // }

    }

}

fn split_multi_info(content: String) {
    let v: Vec<&str> = content.split(';').collect();
    for item in v {
        process_single_info(String::from(item));
    }
}

fn process_single_info(content: String) {
    let mut single_info = TimeIndexInfo::new();
    let v: Vec<&str> = content.split('=').collect();
    if v.len() < 2 {
        return;
    }
    let head_part = String::from(v[0]);
    let mut main_content = String::from(v[1]);

    let _ts_code = head_part.get(head_part.len() - 8..head_part.len());
    main_content.remove(0); // 去除一个引号
    main_content.pop(); // 去除最后一个引号
    // 主体部分
    let main_content_v: Vec<&str> = content.split(',').collect();
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

        let date_time_str = String::from(main_content_v[30]) + "T" + main_content_v[31] + "-08:00";
        single_info.curr_time = DateTime::<Local>::from_str(date_time_str.as_str()).unwrap();
    }
    println!("ret content is {}", content);
}