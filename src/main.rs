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
mod sold;

use log::{error, info, warn};
use std::collections::HashMap;
use chrono::{Local, DateTime, Duration};
// use std::thread::sleep;
use async_std::task::sleep;
use std::str::FromStr;
use async_std::task;
use futures_timer::Delay;

use crate::calculate::{calculate_history_down_s};
use futures::channel::mpsc;
use redis::ConnectionLike;
use crate::cache::AsyncRedisOperation;
use crate::utils::time_utils::{small_step_sleep, my_sleep};
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Row};
use crate::results::{HistoryDown, DBResult};


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

pub async fn calculate_history_dow(mut conn: PoolConnection<MySql>,
                                      stock_codes: Vec<String>,
                                      _code2name_map: HashMap<String, String>) {
    let config = crate::initialize::CONFIG_INFO.get().unwrap();
    let last_days = config.history_down_config.min_history_down_days;
    let min_up_pct = config.history_down_config.min_history_down_up_pct;
    for item in stock_codes {
        // 第一步：查询最近3年多的交易日线信息
        let mut all_close = Vec::<f64>::new();
        let mut sql = String::from("select close from stock_base_info where ts_code='");
        sql = sql + item.as_str() + "' order by trade_date desc limit 900";
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                all_close.push(row.get::<'_, f64, &str>("close"));
            }
        }).await;

        if all_close.is_empty() {
            continue;
        }

        // 第二步：开始计算
        // 计算逻辑：从最新交易记录向旧交易记录遍历，最后一天相比于该天涨了多少
        let mut history_down = HistoryDown::new();
        let mut delta_days = 0;
        let mut delta_pct = 0f64;
        // 最后一天的价格
        let last_day_close = all_close.get(0).unwrap();
        let mut his_down_price = *last_day_close;
        for i in 1..all_close.len() {
            let temp_close = all_close.get(i).unwrap();
            delta_pct = (*last_day_close - *temp_close) / *temp_close;
            if delta_pct <= min_up_pct {
                delta_days = delta_days + 1;
                if *temp_close < his_down_price {
                    his_down_price = *temp_close;
                }
            }
            else {
                break;
            }
        }
        error!("delta days is {}, lastDays is {}", delta_days, last_days);
        if delta_days < last_days {
            continue;
        }
        // 查询最后一天的交易日期
        let mut sql = String::from("select trade_date from stock_base_info where ts_code='");
        sql = sql + item.as_str() + "' order by trade_date desc limit 1";
        let mut trade_date = String::new();
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                trade_date = row.get("trade_date");
            }
        }).await;
        history_down.ts_code = item;
        history_down.his_down_price = his_down_price;
        history_down.in_price = *last_day_close;
        history_down.in_date = trade_date;
        history_down.history_len = delta_days as i64;
        history_down.delta_pct = (history_down.in_price - history_down.his_down_price) /
            history_down.his_down_price;
        error!("yes");

        // sql::insert(&mut conn, history_down).await;
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
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

    task::block_on(async {
        let mut conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
        let codes = vec![String::from("002194.SZ")];
        let mut map= HashMap::<String, String>::new();
        map.insert(String::from("002194.SZ"), String::from("hehe"));
        calculate_history_dow(conn, codes, map).await;
    })
    // let mut vec = vec![1, 2, 3];
    // loop {
    //     if vec.len() <= 0 {
    //         break;
    //     }
    //     println!("{}", vec[0]);
    //     vec.remove(0);
    // }
    // println!("len is {}", vec.len());
    //
    // let handler1 = task::spawn(async {
    //     loop {
    //         my_sleep(Duration::seconds(7200)).await;
    //         error!("sleep finished!!111111111111111111111")
    //     }
    // });
    //
    // let handler2 = task::spawn(async {
    //     loop {
    //         my_sleep(Duration::seconds(14400)).await;
    //         error!("sleep finished!!22222222222222222")
    //     }
    // });
    //
    // let handler3 = task::spawn(async {
    //     loop {
    //         Delay::new(std::time::Duration::from_secs(7200)).await;
    //         error!("futures timer sleep finished!!333333333333333333333333")
    //     }
    // });
    //
    // let handler4 = task::spawn(async {
    //     loop {
    //         Delay::new(std::time::Duration::from_secs(14400)).await;
    //         error!("futures timer sleep finished!!44444444444444444444")
    //     }
    // });
    //
    // task::block_on(async {
    //     handler1.await;
    //     handler2.await;
    //     handler3.await;
    //     handler4.await;
    //
    // });

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

