use futures::executor::{ThreadPoolBuilder, ThreadPool};
use sqlx::pool::PoolOptions;
use futures::executor;
use once_cell::sync::OnceCell;
use sqlx::{MySql, Pool};
use redis::Client;
use crate::config;

static THREAD_POOL: OnceCell<ThreadPool> = OnceCell::new();

static MYSQL_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

static REDIS_POOL: OnceCell<Client> = OnceCell::new();

fn init(mut cx: FunctionContext) -> JsResult<JsBoolean>  {
    let mut final_rst = true;

    let mysql_info = cx.argument::<JsString>(0)?.value();
    let redis_info = cx.argument::<JsString>(1)?.value();

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

    // 如果有回调函数，那么执行一下回调函数
    let args_length = cx.len();
    if args_length > 2 {
        let callback = cx.argument::<JsFunction>(3).unwrap();
        let null = cx.null();
        let args = vec![cx.number(0)];
        callback.call(&mut cx, null, args).unwrap();
    }
    // 返回结果
    Ok(cx.boolean(final_rst))
}