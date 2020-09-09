mod time;
mod sql;
mod results;
mod cache;
mod file;
mod config;
mod calculate;
mod analyzer;
mod initialize;

use chrono::{DateTime, Local};
use std::env;
use std::thread;
use std::time::Duration;
use sqlx::mysql::{MySqlPool, MySqlArguments};
use std::pin::Pin;
use futures::prelude::*;
use futures::future::{ Future, FutureObj };
use futures::executor::{LocalPool, ThreadPool, ThreadPoolBuilder};
use futures::channel::mpsc::{self, Sender, Receiver};
use futures::executor;
use futures::task::{Spawn, SpawnExt};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
use sqlx::mysql::MySqlRow;
use chrono::prelude::*;

use sqlx::{Row, MySql, Pool, Error};
use sqlx::query::Query;
use sqlx::pool::PoolOptions;
use std::fmt::Debug;
use tokio::runtime::{Runtime, Builder};
use std::thread::sleep;
use std::str::FromStr;
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

fn init() {
    let aha = Haha{ value : 34 };
    OHOU.set(aha);

    // let init_block = async {
    //     let pool = MySqlPool::connect("mysql://root:123@localhost:3306/stock").await.unwrap();
    //     MYSQL_POOL.set(pool);
    // };

    // 初始化Redis连接池
    match redis::Client::open("redis://127.0.0.1/") {
        Ok(val) => { REDIS_POOL.set(val).unwrap(); },
        Err(_) => { },
    };

    let runtime = Builder::new()
        .threaded_scheduler()
        .core_threads(8)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
    TOKIO_RUNTIME.set(runtime).unwrap();

    let mut final_rst = true;
    // 初始化数据库连接池
    let init_block = async {
        let mut options = PoolOptions::<MySql>::new();
        options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
        match options.connect("mysql://root:123@localhost:3306/stock").await {
            Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
            Err(_) => { final_rst = false; },
        };
    };

    let mut pool_builder = ThreadPoolBuilder::new();
    pool_builder.after_start(|usize| {
        println!("niubilityle {}", usize);
    });
    let thread_pool = pool_builder.create().unwrap();
    THREAD_POOL.set(thread_pool);
    executor::block_on(init_block);

    let mut temp_value = Vec::<TableMeta>::new();

    common_query("select * from table_meta limit 10", |rows|{
        for row in rows {
            let pk_tablemeta: i32 = row.get("pk_tablemeta");
            let table_name: String = row.get("table_name");
            let is_redis: String = row.get("is_redis");
            println!("hahahahah");
            let tst_val = TableMeta {pk_tablemeta, table_name, is_redis};
            temp_value.push(tst_val);
        }
    });

    for item in &temp_value {
        println!("value is {}", item.pk_tablemeta);
    }
    temp_value.pop();
    for item in &temp_value {
        println!("value is {}", item.pk_tablemeta);
    }
}

fn main() {
    init();
    let s1 = String::from("000001.sz");
    if s1.contains("sz") {
        println!("houhouhouhou什么乱七八糟的");
    }
    let val = s1.get(..6).unwrap();
    let mut new_str = String::from(val);
    new_str.insert_str(0, "sz");
    println!("hah is {}", new_str);

    let vec = vec![String::from("000001.SZ")];
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let runtime = Builder::new()
        .threaded_scheduler()
        .enable_all()
        .core_threads(8)
        // .max_threads(8)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
    //let runtime = Builder::new().threaded_scheduler().enable_all().build().unwrap();
    let join_handler = runtime.spawn(time::fetch_index_info(vec));
    // rt.block_on(time::fetch_index_info(vec));

    // let vec2 = vec![String::from("000002.SZ")];
    // let join_handler2 = runtime.spawn(async {
    //     time::fetch_index_info(vec2).await;
    // });
    let time_str = "2020-09-10T09:09:09-08:00";
    //Local::now();
    match DateTime::<Local>::from_str("2020-09-10T09:09:09-08:00") {
        Ok(val) => println!("ok"),
        Err(err) => println!("err is {}", format!("{:?}", err)),
    }
    let date_time = DateTime::<Local>::from_str("2020-09-10T09:09:09-08:00").unwrap();
    //let date_time = DateTime::<Local>::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap();

    executor::block_on(async {
        join_handler.await;
        // /join_handler2.await;
    });
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

async fn hehe1() {
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