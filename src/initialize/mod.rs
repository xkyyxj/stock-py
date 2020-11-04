use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;
use std::collections::HashMap;
use tokio::runtime::{Runtime, Builder};
use crate::config::Config;

pub(crate) static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub(crate) static REDIS_POOL: OnceCell<Client> = OnceCell::new();

pub(crate) static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) static CONFIG_INFO: OnceCell<Config> = OnceCell::new();

pub fn init(cx: HashMap<String, String>) {
    let mut final_rst = true;

    let mysql_info = cx.get("mysql").unwrap();
    let redis_info = cx.get("redis").unwrap();

    // 初始化基本配置项
    let config_info = Config::new();
    CONFIG_INFO.set(config_info).unwrap();

    // 初始化tokio运行时
    let runtime = Builder::new()
        .threaded_scheduler()
        .enable_all()
        .core_threads(15)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
    TOKIO_RUNTIME.set(runtime).unwrap();

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