mod time;
mod sql;
mod results;
mod cache;
mod file;
mod config;
mod calculate;
mod analyzer;
mod initialize;
mod py_wrapper;
mod utils;
mod selector;
mod simulate;

use std::collections::HashMap;
use crate::py_wrapper::ShortTimeStrategy;
use chrono::{Local, DateTime};
use std::time::Duration;
use std::thread::sleep;
use std::str::FromStr;
use async_std::task;
use calculate::calculate_air_castle;
use crate::calculate::{calculate_history_down, calculate_air_castle_s, parse_table_info, win_calculate};
use futures::channel::mpsc;
use crate::file::read_txt_file;
use async_std::fs::File;
use futures::AsyncWriteExt;

struct A {
    data: i32,
}

struct B {
    a: *mut A,
}

impl B {
    fn set_data(&self, data: i32) {
        unsafe { (*self.a).data = data };
    }
}

fn main() {
    let mut map = HashMap::<String, String>::new();
    map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
    map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
    initialize::init(map);

    // 测试获取实时信息
    // let mut time_fetcher = TimeFetcher{ is_started: false };
    // time_fetcher.clear();
    // time_fetcher.__call__();
    // let mut history_down_ana = HistoryDownAna { is_started: false };
    // history_down_ana.__call__();

    // task::block_on(async {
    //     let pool = crate::initialize::MYSQL_POOL.get().unwrap();
    //     let conn = pool.acquire().await.unwrap();
    //     let ts_codes = vec![String::from("601702.SH")];
    //     let mut map = HashMap::<String, String>::new();
    //     map.insert(String::from("601702.SH"), String::from("hhedada"));
    //     let (mut tx, rx) = mpsc::channel::<u32>(4000);
    //     calculate_air_castle_s(conn, ts_codes, tx, map).await;
    // });
    // task::block_on(calculate_air_castle());
    // task::block_on(async {
    //     let local_time = Local::now();
    //     sync_short_history(&local_time).await;
    // });

    // 短期哦选股的验证逻辑
    // let mut short_time_select = ShortTimeStrategy::new();
    // short_time_select.__call__();

    let future1 = async {
        println!("first");
        sleep(Duration::from_secs(10));
        println!("first_end;");
    };

    let future2 = async {
        println!("second");
        sleep(Duration::from_secs(10));
        println!("second_end;");
    };

    let future3 = async {
        futures::join!(future1, future2);
    };

    task::block_on(future3);



    // 文件读取以及解析验证
    // task::block_on(async {
    //     win_calculate().await;
    // });

    // 低值计算验证
    // task::block_on(calculate_history_down());

    match DateTime::<Local>::from_str("2020-11-02T15:00:03 +08:00") {
        Ok(_val) => println!("ok， val is {}", _val),
        Err(err) => println!("err is {}", format!("{:?}", err)),
    }
    // match DateTime::<Local>::from_str("2020-09-18 23:05:33.299294600 +08:00") {
    //     Ok(_val) => println!("ok， val is {}", _val),
    //     Err(err) => println!("err is {}", format!("{:?}", err)),
    // }
    // let _date_time = DateTime::<Local>::from_str("2020-09-10T09:09:09-08:00").unwrap();
    // let date_time = DateTime::<FixedOffset>::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap();
    // println!("val is {}", date_time);
    // executor::block_on(async {
    //     join_handler.await;
    //     // /join_handler2.await;
    // });
    sleep(Duration::from_secs(100000));

}

