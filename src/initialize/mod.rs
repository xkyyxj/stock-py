use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;
use std::collections::HashMap;
use crate::config::Config;
use crate::utils::Taskbar;
use log::{error, info, warn};
use crate::utils::time_utils::TimeCheck;
use async_std::sync::{Mutex, Arc};
use std::thread;

pub static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub static REDIS_POOL: OnceCell<Client> = OnceCell::new();

pub static CONFIG_INFO: OnceCell<Config> = OnceCell::new();

pub static TASKBAR_TOOL: OnceCell<Taskbar> = OnceCell::new();

pub static TIME_CHECK: OnceCell<Arc<TimeCheck>> = OnceCell::new();

pub fn init(cx: HashMap<String, String>) {
    // 初始化日志组件
    log4rs::init_file("./log.yaml", Default::default()).unwrap();
    warn!("Log initialized fished!");

    let mut final_rst = true;

    let mysql_info = cx.get("mysql").unwrap();
    let redis_info = cx.get("redis").unwrap();

    // 初始化基本配置项
    let config_info = Config::new();
    CONFIG_INFO.set(config_info).unwrap();

    // 初始化数据库连接池
    let mysql_init = async {
        let mut options = PoolOptions::<MySql>::new();
        options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
        match options.connect(mysql_info.as_str()).await {
            Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
            Err(err) => { error!("err is {}", format!("{:?}", err)) },
        };
    };
    executor::block_on(mysql_init);
    warn!("Mysql connection initialized fished!");

    // 初始化Redis连接池
    match redis::Client::open(redis_info.as_str()) {
        Ok(val) => { REDIS_POOL.set(val).unwrap(); },
        Err(_) => { final_rst = false; },
    };
    warn!("Redis connection initialized fished!");

    // 初始化windows任务栏（不管了）
    TASKBAR_TOOL.set(Taskbar::new()).unwrap();

    let time_check = Arc::new(TimeCheck::new());
    let time_check2 = time_check.clone();
    thread::spawn(move || {time_check.start();});
    TIME_CHECK.set(time_check2);
}