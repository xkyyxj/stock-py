use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;
use std::collections::HashMap;
use crate::config::Config;
use crate::utils::Taskbar;

pub static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub static REDIS_POOL: OnceCell<Client> = OnceCell::new();

pub static CONFIG_INFO: OnceCell<Config> = OnceCell::new();

pub static TASKBAR_TOOL: OnceCell<Taskbar> = OnceCell::new();

pub fn init(cx: HashMap<String, String>) {
    let mut final_rst = true;

    let mysql_info = cx.get("mysql").unwrap();
    let redis_info = cx.get("redis").unwrap();

    // 初始化基本配置项
    let config_info = Config::new();
    CONFIG_INFO.set(config_info).unwrap();

    // 初始化数据库连接池
    let mysql_init = async {
        println!("mysql info is {}", mysql_info);
        let mut options = PoolOptions::<MySql>::new();
        options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
        match options.connect(mysql_info.as_str()).await {
            Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
            Err(err) => { println!("err is {}", format!("{:?}", err)) },
        };
    };
    executor::block_on(mysql_init);

    // 初始化Redis连接池
    match redis::Client::open(redis_info.as_str()) {
        Ok(val) => { REDIS_POOL.set(val).unwrap(); },
        Err(_) => { final_rst = false; },
    };

    // 初始化windows任务栏（不管了）
    TASKBAR_TOOL.set(Taskbar::new()).unwrap();
}