use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;
use std::collections::HashMap;
use tokio::runtime::{Runtime, Builder};
use crate::config::Config;
use crate::utils::Taskbar;

pub(crate) static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub(crate) static REDIS_POOL: OnceCell<Client> = OnceCell::new();

pub(crate) static TOKIO_RUNTIME: OnceCell<Runtime> = OnceCell::new();

pub(crate) static CONFIG_INFO: OnceCell<Config> = OnceCell::new();

pub(crate) static TASKBAR_TOOL: OnceCell<Taskbar> = OnceCell::new();

pub fn init(cx: HashMap<String, String>) {
    let mut final_rst = true;

    let mysql_info = cx.get("mysql").unwrap();
    let redis_info = cx.get("redis").unwrap();

    // 初始化基本配置项
    let config_info = Config::new();
    CONFIG_INFO.set(config_info).unwrap();

    // 初始化tokio运行时
    let runtime = Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(4)
        .max_threads(512)
        .thread_name("my-custom-name")
        .thread_stack_size(3 * 1024 * 1024)
        .build()
        .unwrap();
    TOKIO_RUNTIME.set(runtime).unwrap();

    // 初始化Redis连接池
    match redis::Client::open(redis_info.as_str()) {
        Ok(val) => { REDIS_POOL.set(val).unwrap(); },
        Err(_) => { final_rst = false; },
    };

    // 初始化windows任务栏（不管了）
    TASKBAR_TOOL.set(Taskbar::new()).unwrap();

    // 初始化数据库连接池
    let tokio_runtime = TOKIO_RUNTIME.get().unwrap();
    // tokio_runtime.block_on(init_block);
    // tokio_runtime.spawn_blocking()
    let join_handler = tokio_runtime.spawn(init_mysql_pool(String::from(mysql_info)));
    executor::block_on(join_handler);

}

async fn init_mysql_pool(conn_info: String) {
    let mut options = PoolOptions::<MySql>::new();
    options = options.max_connections(config::MYSQL_MAX_CONNECTION as u32);
    match options.connect(conn_info.as_str()).await {
        Ok(val) => { MYSQL_POOL.set(val).unwrap(); },
        Err(_) => {  },
    };
}