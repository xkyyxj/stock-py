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

use chrono::{DateTime, Local, FixedOffset, TimeZone};



use sqlx::mysql::{MySqlArguments};
use async_std::task;


use futures::executor::{ThreadPool, ThreadPoolBuilder};

use futures::executor;
use futures::task::{SpawnExt};

use once_cell::sync::OnceCell;
use sqlx::mysql::MySqlRow;


use sqlx::{Row, MySql, Pool};
use sqlx::query::Query;
use sqlx::pool::PoolOptions;

use tokio::runtime::{Runtime, Builder};
use tokio::task::JoinHandle;

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
// use std::marker::Pinned;
// use std::sync::{Arc, Mutex};
// use mysql::*;
// use mysql::prelude::*;

// struct PinnedTest {
//     value: u32,
//     _pin: Pinned,
// }

struct Haha {
    value: u32
}

pub async fn insert_into() {
    let pool = MYSQL_POOL.get().unwrap();
    sqlx::query("insert into table_meta(table_name, is_redis) values('hehe', 'N')").execute(pool).await;
}

impl Drop for Haha {
    fn drop(&mut self) {
        println!("hahah , droped");
    }
}

pub fn common_query(sql: &str, mut f: impl FnMut(&Vec<MySqlRow>)) {
    let conn = MYSQL_POOL.get().unwrap();
    let query_fut = sqlx::query(sql).fetch_all(conn);
    let all_rows = executor::block_on(query_fut).unwrap();
    f(&all_rows);
}

static aha2: Haha = Haha {value : 45};

static OHOU: OnceCell<Haha> = OnceCell::new();

static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

static REDIS_POOL: OnceCell<redis::Client> = OnceCell::new();

pub trait Result {
    fn insert<'a>(&self) -> Query<'a, MySql, MySqlArguments>;

    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments>;
}

struct TableMeta {
    pk_tablemeta: i32,
    table_name: String,
    is_redis: String
}

impl Result for TableMeta {
    fn insert<'a>(&self) -> Query<'a, MySql, MySqlArguments> {
        sqlx::query("insert into table_meta(table_name, is_redis) values(?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.table_name);
        query.bind(&self.is_redis)
    }
}

fn insert(val: impl Result) {
    let mut query = val.insert();
    query = val.bind(query);
    let conn = MYSQL_POOL.get().unwrap();
    match executor::block_on(query.execute(conn)) {
        Ok(_) => println!("ok"),
        Err(err) => println!("err is {}", format!("{:?}", err)),
    }
}

// fn init() {
//     let aha = Haha{ value : 34 };
//     OHOU.set(aha);
//
//     // let init_block = async {
//     //     let pool = MySqlPool::connect("mysql://root:123@localhost:3306/stock").await.unwrap();
//     //     MYSQL_POOL.set(pool);
//     // };
//
//     // 初始化Redis连接池
//     match redis::Client::open("redis://127.0.0.1/") {
//         Ok(val) => { REDIS_POOL.set(val).unwrap(); },
//         Err(_) => { },
//     };
//
//     let runtime = Builder::new()
//         .threaded_scheduler()
//         .core_threads(8)
//         .thread_name("my-custom-name")
//         .thread_stack_size(3 * 1024 * 1024)
//         .build()
//         .unwrap();
//     TOKIO_RUNTIME.set(runtime).unwrap();
//
//     let mut final_rst = true;
//     // 初始化数据库连接池
//     let init_block = async {
//         let mut options = PoolOptions::<MySql>::new();
//         options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
//         match options.connect("mysql://root:123@localhost:3306/stock").await {
//             Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
//             Err(_) => { final_rst = false; },
//         };
//     };
//
//     let mut pool_builder = ThreadPoolBuilder::new();
//     pool_builder.after_start(|usize| {
//         println!("niubilityle {}", usize);
//     });
//     let thread_pool = pool_builder.create().unwrap();
//     THREAD_POOL.set(thread_pool);
//     executor::block_on(init_block);
//
//     let mut temp_value = Vec::<TableMeta>::new();
//
//     common_query("select * from table_meta limit 10", |rows|{
//         for row in rows {
//             let pk_tablemeta: i32 = row.get("pk_tablemeta");
//             let table_name: String = row.get("table_name");
//             let is_redis: String = row.get("is_redis");
//             println!("hahahahah");
//             let tst_val = TableMeta {pk_tablemeta, table_name, is_redis};
//             temp_value.push(tst_val);
//         }
//     });
//
//     for item in &temp_value {
//         println!("value is {}", item.pk_tablemeta);
//     }
//     temp_value.pop();
//     for item in &temp_value {
//         println!("value is {}", item.pk_tablemeta);
//     }
// }
// fn main() {
//     let mut map = HashMap::<String, String>::new();
//     map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
//     map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
//     initialize::init(map);
//
//     let mut fetch = crate::py_wrapper::TimeFetcher{is_started: false};
//     fetch.__call__();
//     sleep(Duration::from_secs(100000));
// }

// fn test_aysnc1() {
//     let client = crate::initialize::REDIS_POOL.get().unwrap();
//     let mut connection = client.get_async_connection().await.unwrap();
// }

fn air_castle_cal() {
    let ts_codes = vec![String::from("000001.SZ")];
    let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    let join_handler = tokio_runtime.spawn(async{
        calculate_air_castle().await;
    });
    executor::block_on(async {
        join_handler.await;
    });
}

fn main() {
    // show_win_toast(String::from("123"), String::from("hehedada"));
    // let mut toast = Taskbar::new();
    // let ten_millis2 = std::time::Duration::from_secs(2);
    // thread::sleep(ten_millis2);
    // toast.show_win_toast(String::from("您有新的待选股票啦"), String::from("hehedada"));
    //
    // let ten_millis3 = std::time::Duration::from_secs(2);
    // thread::sleep(ten_millis3);
    // toast.show_win_toast(String::from("6666"), String::from("7777"));
    // let ten_millis = std::time::Duration::from_secs(30);
    // thread::sleep(ten_millis);

    let mut map = HashMap::<String, String>::new();
    map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
    map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
    initialize::init(map);

    // let ts_codes = vec![String::from("000001.SZ")];
    // let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    // let join_handler = tokio_runtime.spawn(async{
    //     // let mut history_down = HistoryDownAnalyzer::new();
    //     // history_down.analyze().await;
    //     // fetch_index_info(ts_codes).await;
    //     let mut air_castle_val = AirCastle::new();
    //     air_castle_val.ts_code = String::from("123213");
    //     air_castle_val.in_price = 0 as f64;
    //     air_castle_val.in_date = String::from("234234");
    //     air_castle_val.up_days = 0;
    //     air_castle_val.ave_day_up_pct = 0 as f64;
    //     air_castle_val.up_pct = 0 as f64;
    //     let mut conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
    //     sql::insert(&mut conn, air_castle_val).await;
    // });
    // executor::block_on(async {
    //     join_handler.await;
    // });
    // air_castle_cal();

    // 尝试select
    // let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    // tokio_runtime.spawn(async {
    //     let selector = ShortTimeSelect::new();
    //     selector.select().await;
    // });

    // 测试获取实时信息
    let mut time_fetcher = TimeFetcher{ is_started: false };
    time_fetcher.clear();
    time_fetcher.__call__();
    let mut history_down_ana = HistoryDownAna { is_started: false };
    history_down_ana.__call__();


    let mut short_time_select = ShortTimeStrategy::new();
    short_time_select.__call__();

    task::spawn(async {
        // some work here
    });

    // 测试查询history_down
    // let rst1 = StockBaseInfo::query(Some("ts_code='000001.SZ'".parse().unwrap()));
    // let rst = HistoryDown::query(None);
    sleep(Duration::from_secs(2000));
    // test_aysnc1();
    // let temp_future = async {
    //     let mut async_conn = redis_client.get_async_connection().await.unwrap();
    //     async_conn.set("123", "123");
    // };
    // executor::block_on(temp_future);
    // // init();
    // let s1 = String::from("000001.sz");
    // if s1.contains("sz") {
    //     println!("houhouhouhou什么乱七八糟的");
    // }
    // let val = s1.get(..6).unwrap();
    // let mut new_str = String::from(val);
    // new_str.insert_str(0, "sz");
    // println!("hah is {}", new_str);
    // let temp_str = String::from("hehe;dada");
    // let temp_v: Vec<&str> = temp_str.split(';').collect();
    // for item in temp_v {
    //     println!("v is {}", item);
    // }
    // let mut str1 = String::new();
    // {
    //     let str2 = String::from("123");
    //     str1 = str1.add(str2.as_str());
    // }
    // println!("str is {}", str1);
    //
    // let mut temp = crate::results::TimeIndexBatchInfo::new();
    // temp.ts_code = String::from("123123");
    // // println!("ts code is {}", temp.ts_code.as_str());
    // temp.ts_name = String::from("hehedada");
    // println!("hehedada {}", !temp.ts_code.is_empty());
    // //println!("ts code2 is {}", temp.ts_code.as_str());
    // let temp1 = crate::results::TimeIndexInfo::new();
    // temp.add_single_info(&temp1);
    // println!("{}", temp);
    // println!("after {}", temp.ts_code);
    //
    // let value2 = async {
    //     println!("hehedada");
    // };
    // let value: BoxFuture<()> = Box::pin(hehe1()); // 这是正确的写法
    // let value: Box<dyn Future<Output=()>> = Box::new(hehe1()); // 这是错误的写法
    // executor::block_on(value);

    // executor::block_on(calculate::calculate_history_down());

    //test2222();
    // let vec = vec![String::from("000001.SZ"), String::from("000002.SZ")];
    // //let _rt = tokio::runtime::Runtime::new().unwrap();
    // let runtime = Builder::new()
    //     .threaded_scheduler()
    //     .enable_all()
    //     .core_threads(8)
    //     // .max_threads(8)
    //     .thread_name("my-custom-name")
    //     .thread_stack_size(3 * 1024 * 1024)
    //     .build()
    //     .unwrap();
    // //let runtime = Builder::new().threaded_scheduler().enable_all().build().unwrap();
    // // let join_handler = runtime.spawn(time::fetch_index_info(vec));
    // runtime.spawn(time::fetch_index_info(vec));
    //rt.block_on(time::fetch_index_info(vec));
    // sleep(Duration::from_secs(10000));

    // let vec2 = vec![String::from("000002.SZ")];
    // let join_handler2 = runtime.spawn(async {
    //     time::fetch_index_info(vec2).await;
    // });
    // let _time_str = "2020-09-10T09:09:09-08:00";
    // //Local::now();
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
    // sleep(Duration::from_secs(1000));
    // init();
    // let meta = TableMeta {
    //     pk_tablemeta: 0,
    //     table_name: "123".to_string(),
    //     is_redis: "Y".to_string()
    // };
    // let date_time = Local::now();
    // //insert(meta);
    // //println!("curr date is {}", date_time.format("%Y%m%d").to_string());
    // executor::block_on(calculate::calculate_in_low());
    //executor::block_on(insert_into());
    // let table_meta = TableMeta{ pk_tablemeta: 23, table_name: String::from("234"), is_redis: String::from("67")};
    // insert(table_meta);
    // let fut = hehe1();
    // let thread_pool = THREAD_POOL.get().unwrap();
    // thread_pool.spawn(fut);
    // let all_rows = executor::block_on(fut).unwrap();
    // for row in all_rows {
    //     let val: String = row.get("ts_code");
    //     println!("val is {}", val);
    // }
}

// fn test333<'a>(target_fun: impl Fn() -> BoxFuture<'a, ()>) {
//     hehe1()
// }

// fn test44() -> impl Future<Output=()> {
//
// }

fn test2222() {
    // 下面一段代码报错了
    let columns = vec!["ts_code"];
    let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    let stock_codes_rows = executor::block_on(sql::query_stock_list(&columns, "")).unwrap();
    let mut count = 0;
    let mut each_thread_codes = Vec::<String>::new();
    // each_thread_codes.push(String::from("000001.SZ"));
    // each_thread_codes.push(String::from("000002.SZ"));
    // tokio_runtime.spawn(crate::time::fetch_index_info(each_thread_codes));
    // each_thread_codes = Vec::<String>::new();
    // each_thread_codes.push(String::from("002668.SZ"));
    // each_thread_codes.push(String::from("600588.SH"));
    // tokio_runtime.spawn(crate::time::fetch_index_info(each_thread_codes));
    //tokio_runtime.spawn(crate::time::fetch_index_info(each_thread_codes));
    let mut join_handlers = Vec::<JoinHandle<()>>::new();
    for row in &stock_codes_rows {
        let ts_code: String = row.get("ts_code");
        each_thread_codes.push(ts_code);
        count = count + 1;
        if count == 330 {
            join_handlers.push(tokio_runtime.spawn(crate::time::fetch_index_info(each_thread_codes)));
            each_thread_codes = Vec::<String>::with_capacity(330);
            count = 0;
        }
    }
    executor::block_on(async {
        for item in join_handlers {
            item.await;
        }
    })

    // let tokio_runtime = Builder::new()
    //     .threaded_scheduler()
    //     .enable_all()
    //     .core_threads(8)
    //     .thread_name("my-custom-name")
    //     .thread_stack_size(3 * 1024 * 1024)
    //     .build()
    //     .unwrap();
    // let mut each_thread_codes = Vec::<String>::new();
    // each_thread_codes.push(String::from("000001.SZ"));
    // each_thread_codes.push(String::from("000002.SZ"));
    // tokio_runtime.spawn(time::fetch_index_info(each_thread_codes));
    // sleep(Duration::from_secs(200000));

    // 下面一段代码正常工作
    // let vec = vec![String::from("000001.SZ"), String::from("000002.SZ")];
    // //let _rt = tokio::runtime::Runtime::new().unwrap();
    // let runtime = Builder::new()
    //     .threaded_scheduler()
    //     .enable_all()
    //     .core_threads(8)
    //     // .max_threads(8)
    //     .thread_name("my-custom-name")
    //     .thread_stack_size(3 * 1024 * 1024)
    //     .build()
    //     .unwrap();
    // //let runtime = Builder::new().threaded_scheduler().enable_all().build().unwrap();
    // // let join_handler = runtime.spawn(time::fetch_index_info(vec));
    // runtime.spawn(time::fetch_index_info(vec));
    // sleep(Duration::from_secs(10000));

    // for row in &stock_codes_rows {
    //     let ts_code: String = row.get("ts_code");
    //     each_thread_codes.push(ts_code);
    //     count = count + 1;
    //     if count == 330 {
    //         tokio_runtime.spawn(crate::time::fetch_index_info(each_thread_codes));
    //         each_thread_codes = Vec::<String>::with_capacity(330);
    //         count = 0;
    //     }
    // }
}

async fn hehe1() -> () {
    let conn = MYSQL_POOL.get().unwrap();
    sqlx::query(r#"select * from stock_base_info limit 10"#).fetch_all(conn).await.unwrap();
}

// ---------------- 以下是哦吼测试

// struct test {
//     value: String,
// }
//
// pub trait lala {
//     fn yeye(&self) -> usize;
// }
//
// impl lala for test {
//     fn yeye(&self) -> usize {
//         7
//     }
// }
//
// fn test_move_vec(value: &Vec<&str>) {
//     for item in value {
//         println!("value is {}", item);
//     }
// }
//
// fn main() {
//     let hehe = vec!["44","33"];
//     test_move_vec(&hehe);
//     for item in hehe {
//         println!("xixi : {}", item);
//     }
//
//     init();
//     //let mut pool = LocalPool::new();
//     //let spawner = pool.spawner();
//     //let test_future = test2();
//     //let test_f_box = FutureObj::new(Box::new(test_future));
//
//     //spawner.spawn_obj(test_f_box);
//     //pool.run();
//
//     let (mut tx, mut rx) = mpsc::channel::<u32>(100);
//     let wrap_ft = async {
//         let thread_pool = THREAD_POOL.get().unwrap();
//         thread_pool_test(thread_pool, tx, rx).await
//     };
//     //tokio::spawn(test_future);
//
//     //test_one.await?;
//     let retVal = executor::block_on(wrap_ft);
//     for item in retVal {
//         println!("value is {}", item);
//     }
//
//     println!("hahahahahhaha");
//     //thread::sleep(Duration::from_secs(3));
//     // executor::block_on(test());
// }
//
// async fn thread_pool_test(thread_pool: &ThreadPool, mut tx: Sender<u32>, rx: Receiver<u32>) -> Vec<u32> {
//     let test_future = test2(tx);
//     let test_f_box = FutureObj::new(Box::new(test_future));
//     let pool = THREAD_POOL.get().unwrap();
//     pool.spawn_obj_ok(test_f_box);
//     rx.collect::<Vec<u32>>().await
//     //thread_pool.run();
// }
//
// fn test() ->  impl Future<Output = ()> {
//     async {
//         let pool = MySqlPool::connect("mysql://root:123@localhost:3306/stock").await.unwrap();
//         let rows = sqlx::query(
//         r#"
//     SELECT * from stock_base_info limit 10
//         "#).fetch_all(&pool).await.unwrap();
//
//         let test1 = test{ value : String::from("7777") };
//         test1.yeye();
//         for temp_row in rows {
//             let ts_code : String = temp_row.get("ts_code");
//             println!("ts_code is {}", ts_code);
//         }
//
//         let mut rows2 = sqlx::query(
//             r#"
//         SELECT * from stock_base_info limit 10
//             "#).fetch(&pool);
//         while let Some(item) = rows2.next().await {
//             let temp_row_item = item.unwrap();
//             let ts_code2 : String = temp_row_item.get("ts_code");
//             println!("ts_code is {}", ts_code2);
//         }
//
//         //Ok(())
//
//         //ok(Ok(()))
//     }
// }
//
// async fn test2(mut tx: mpsc::Sender<u32>) {
//     let pool = MYSQL_POOL.get().unwrap();
//     let rows = sqlx::query(
//     r#"
// SELECT * from stock_base_info limit 10
//     "#).fetch_all(pool).await.unwrap();
//
//     let test1 = test{ value : String::from("7777") };
//     test1.yeye();
//     for temp_row in rows {
//         let ts_code : String = temp_row.get("ts_code");
//         println!("ts_code is {}", ts_code);
//     }
//
//     let mut rows2 = sqlx::query(
//         r#"
//     SELECT * from stock_base_info limit 10
//         "#).fetch(pool);
//     while let Some(item) = rows2.next().await {
//         let temp_row_item = item.unwrap();
//         let ts_code2 : String = temp_row_item.get("ts_code");
//         println!("ts_code is {}", ts_code2);
//     }
//     tx.send(23);
// }

//--------------------------------------------- 以上是哦吼测试

// fn test() ->  impl Future<Output = Result<(), sqlx::Error>> {
//     async {
//         let pool = MySqlPool::connect("mysql://root:123@localhost:3306/stock").await?;
//         let rows = sqlx::query(
//         r#"
//     SELECT * from stock_base_info limit 10
//         "#).fetch_all(&pool).await?;

//         let test1 = test{ value : String::from("7777") };
//         test1.yeye();
//         for temp_row in rows {
//             let ts_code : String = temp_row.get("ts_code");
//             println!("ts_code is {}", ts_code);
//         }

//         let mut rows2 = sqlx::query(
//             r#"
//         SELECT * from stock_base_info limit 10
//             "#).fetch(&pool);
//         while let Some(item) = rows2.next().await {
//             let temp_row_item = item.unwrap();
//             let ts_code2 : String = temp_row_item.get("ts_code");
//             println!("ts_code is {}", ts_code2);
//         }

//         Ok(())

//         //ok(Ok(()))
//     }
// }

// #[async_std::main] // or #[tokio::main]
// #[paw::main]
// async fn main() -> Result<(), sqlx::Error> {

//     // let pinned_test = PinnedTest {
//     //     value: 32,
//     //     _pin: Pinned,
//     // }

//     // let test_box = Box::new(pinned_test);

//     // Create a connection pool
//     let pool = MySqlPool::connect("mysql://root:123@localhost:3306/stock").await?;

//     // Make a simple query to return the given parameter
// //     let rows = sqlx::query(
// //         r#"
// // SELECT * from stock_base_info limit 10
// //         "#).fetch(&pool);

//     let rows = sqlx::query(
//     r#"
// SELECT * from stock_base_info limit 10
//     "#).fetch_all(&pool).await?;
//     //let real_rows = Pin::into_inner(rows).poll();
//     //rows.poll_next();

//     //let retRows = rows.get_ref();
//     // let box1 = Box::new(1);
//     // let pin1 = Pin::new(box1);
//     // let box2 = Pin::into_inner(pin1);

//     //println!("ts_code is {}", row);
//     // while let Some(row) = rows.next().await? {
//     //     let id: i32 = row.get(0);
//     //     let table_name: String = row.get(1);
//     //     let is_redis: String = row.get(2);
//     //     println!("row : {}, {}, {}", id, table_name, is_redis);
//     // };
//        // println!("123 : {}", rows);

//        let test1 = test{ value : String::from("7777") };
//        test1.yeye();
//     for temp_row in rows {
//         //let hehe = Box::new(temp_row);
//         //temp_row.ohou();
//         //temp_row.get("te'");
//         let ts_code : String = temp_row.get("ts_code");
//         println!("ts_code is {}", ts_code);
//     }

//     let rows2 = sqlx::query(
//         r#"
//     SELECT * from stock_base_info limit 10
//         "#).fetch(&pool);

//     Ok(())
// }

// struct TableMeta {
//     pk_tablemeta: i32,
//     table_name: String,
//     is_redis: String
// }

// fn main() {
//     let pool = Pool::new("mysql://root:123@localhost:3306/stock").unwrap();
//     let mut conn = pool.get_conn().unwrap();

//     let selected_val = conn.query_map(
//         "select * from table_meta",
//         |(pk_tablemeta, table_name, is_redis)| {
//             TableMeta {
//                 pk_tablemeta, table_name, is_redis
//             }
//         }
//     );
// }