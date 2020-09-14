use futures::executor::{ThreadPoolBuilder, ThreadPool};
use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;
use std::collections::HashMap;

pub(crate) static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

pub(crate) static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub(crate) static REDIS_POOL: OnceCell<Client> = OnceCell::new();

fn init(mut cx: HashMap<String, String>) {
    let mut final_rst = true;

    let mysql_info = cx.get("mysql").unwrap();
    let redis_info = cx.get("redis").unwrap();

    // 初始化线程池
    let mut pool_builder = ThreadPoolBuilder::new();
    match pool_builder.create() {
        Ok(val) => { THREAD_POOL.set(val).unwrap(); },
        Err(_) => { final_rst = false; },
    };

    // 初始化数据库连接池
    let init_block = async {
        let mut options = PoolOptions::<MySql>::new();
        options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
        match options.connect(mysql_info.as_str()).await {
            Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
            Err(_) => { final_rst = false; },
        };
    };
    executor::block_on(init_block);

    // 初始化Redis连接池
    match redis::Client::open(redis_info.as_str()) {
        Ok(val) => { REDIS_POOL.set(val).unwrap(); },
        Err(_) => { final_rst = false; },
    };

}