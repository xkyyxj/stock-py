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

use chrono::{DateTime, Local, FixedOffset, TimeZone};



use sqlx::mysql::{MySqlArguments};
use async_std::task;

use futures::task::{SpawnExt};

use once_cell::sync::OnceCell;
use sqlx::mysql::MySqlRow;


use sqlx::{Row, MySql, Pool};
use sqlx::query::Query;
use sqlx::pool::PoolOptions;


use std::str::FromStr;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use crate::results::{DBResult, HistoryDown, StockBaseInfo};
use std::ops::Add;
use futures::future::{Future, BoxFuture};
use redis::{AsyncCommands, Commands};
use crate::time::{fetch_index_info, INDEX_SUFFIX};
use crate::utils::{Taskbar};
use crate::results::{AirCastle};
use std::thread;
use crate::py_wrapper::{HistoryDownAna, TimeFetcher, ShortTimeStrategy};
use crate::analyzer::HistoryDownAnalyzer;
use crate::cache::AsyncRedisOperation;
use crate::calculate::calculate_air_castle;
use crate::selector::ShortTimeSelect;
use async_std::task::JoinHandle;

fn air_castle_cal() {
    let ts_codes = vec![String::from("000001.SZ")];
    let join_handler = task::spawn(async{
        calculate_air_castle().await;
    });
    task::block_on(async {
        join_handler.await;
    });
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


    let mut short_time_select = ShortTimeStrategy::new();
    short_time_select.__call__();

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

