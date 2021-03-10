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

use log::{error, info, warn};
use std::collections::HashMap;
use chrono::{Local, DateTime, Duration};
// use std::thread::sleep;
use async_std::task::sleep;
use std::str::FromStr;
use async_std::task;

use crate::calculate::{calculate_history_down_s};
use futures::channel::mpsc;
use redis::ConnectionLike;
use crate::cache::AsyncRedisOperation;
use crate::utils::time_utils::{small_step_sleep, my_sleep};


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
    // log4rs::init_file("log.yaml", Default::default()).unwrap();
    // initialize();
    // let handler = task::spawn(async {
    //     let duration = Duration::seconds(10000);
    //     small_step_sleep(&duration).await;
    // });
    // task::block_on(handler);

    let mut map = HashMap::<String, String>::new();
    map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
    map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
    initialize::init(map);
    let mut vec = vec![1, 2, 3];
    loop {
        if vec.len() <= 0 {
            break;
        }
        println!("{}", vec[0]);
        vec.remove(0);
    }
    println!("len is {}", vec.len());

    task::block_on(async {
        loop {
            my_sleep(Duration::seconds(7200)).await;
            error!("sleep finished!!")
        }
    });

    // loop {
    //     // std::thread::sleep(Duration::from_secs(1));
    //     // info!("booting up");
    //     task::spawn(async {
    //         let duration = Duration::seconds(10000);
    //         small_step_sleep(&duration).await;
    //     });
    //
    // }
    // let mut map = HashMap::<String, String>::new();
    // map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
    // map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
    // initialize::init(map);
    // info!("hehehehehe");
    //
    //
    // task::block_on(async {
    //     // let conn = crate::initialize::MYSQL_POOL.get().unwrap();
    //     // sqlx::query("select * from stock_list where ts_code='000001.SZ'").fetch_all(conn).await;
    //     let mut conn = AsyncRedisOperation::new().await;
    //     let mut val = 1;
    //     loop {
    //         println!("1111111");
    //         info!("hehehehehe");
    //         val += 1;
    //         conn.set("123", val.to_string()).await;
    //         // sleep(Duration::from_secs(2000));
    //         sleep(Duration::from_secs(3)).await;
    //         println!("2222222");
    //     }
    // });

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

    // let a = A{ data: 0 };
    // let test = Arc::new(Mutex::new(a));
    // let test2 = test.clone();
    // task::spawn(async move {
    //     println!("a start");
    //     let mut testa1 = &mut *test2.lock().await;
    //     println!("before sleep111");
    //     async_std::task::sleep(Duration::from_secs(5)).await;
    //     testa1.data = 100;
    //     println!("a end  val is {}", testa1.data);
    // });
    //
    // let test3 = test.clone();
    // task::spawn(async move {
    //     println!("b start");
    //     let mut testa1 = &mut *test3.lock().await;
    //     println!("before sleep222");
    //     async_std::task::sleep(Duration::from_secs(5)).await;
    //     testa1.data = 200;
    //     println!("b end  val is {}", testa1.data);
    // });

    // let mut selector = CommonSelectStrategy::new();
    // selector.__call__();
    // task::block_on(async {
    //     let mut select = EMASelect::new().await;
    //     select.initialize().await;
    //
    //     let (mut tx, rx) = mpsc::unbounded::<CommonSelectRst>();
    //     select.select(tx).await;
    //     let all_common_rst = rx.collect::<Vec<CommonSelectRst>>().await;
    //
    //     let local: DateTime<Local> = Local::now();
    //     let year = local.date().year();
    //     let month = local.date().month();
    //     let day = local.date().day();
    //     let temp_curr_time = Local.ymd(year, month, day).and_hms_milli(11, 29, 59, 0);
    //     let mut rst_process = CommonTimeRstProcess::new();
    //     rst_process.process(all_common_rst.get(0).unwrap(), &temp_curr_time).await;
    // });

    // task::block_on(async {
    //     let conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
    //     let (tx, _rx) = mpsc::channel::<u32>(4000);
    //     let ts_codes = vec![String::from("000429.SZ"), String::from("300800.SZ")];
    //     let map = HashMap::<String, String>::new();
    //     calculate_history_down_s(conn, ts_codes, tx, map).await;
    //     // calculate_air_castle().await;
    // });

    // 文件读取以及解析验证
    // task::block_on(async {
    //     win_calculate().await;
    // });

    // 低值计算验证
    // task::block_on(calculate_history_down());

    // match DateTime::<Local>::from_str("2020-11-02T15:00:03 +08:00") {
    //     Ok(_val) => println!("ok， val is {}", _val),
    //     Err(err) => println!("err is {}", format!("{:?}", err)),
    // }
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
    // sleep(Duration::from_secs(100000));

}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     log4rs::init_file("./log.yaml", Default::default()).unwrap();
//     tokio::spawn(async move {
//         error!("start");
//         tokio::time::sleep(Duration::from_secs(7200));
//         error!("end");
//     });
// }

