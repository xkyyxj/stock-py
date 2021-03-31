use jni::JNIEnv;
use jni::objects::{JClass, JMap, JObject, JString};
use combine::lib::collections::HashMap;

use futures::{executor};
use async_std::{task};
use crate::sql;
use crate::time::fetch_index_info;
use sqlx::Row;
use crate::selector::AllSelectStrategy;
use crate::calculate::calculate_history_down;
use crate::sold::TrackSold;
use crate::initialize::init;
use jni::errors::Error;
use jni::signature::JavaType;

// 每个线程负责拉取的股票数量
static EACH_THREAD_FETCH_NUM: usize = 330;

#[no_mangle]
pub extern "system" fn Java_com_cassiopeia_rust_RustAPI_initialize(env: JNIEnv, class: JClass, info: JObject) {
    let mysql_key = env.new_string("mysql").unwrap();
    let redis_key = env.new_string("redis").unwrap();

    let class = env.auto_local(env.find_class("java/util/Map").unwrap());
    let get = env.get_method_id(&class, "get", "(Ljava/lang/Object;)Ljava/lang/Object;").unwrap();
    let mysql_rst = env.call_method_unchecked(info, get, JavaType::Object("java/lang/Object".into()),
                              &[JObject::from(mysql_key).into()]);
    let mysql_value = mysql_rst.unwrap().l().unwrap();
    let redis_rst = env.call_method_unchecked(info, get, JavaType::Object("java/lang/Object".into()),
                                              &[JObject::from(redis_key).into()]);
    let redis_value = redis_rst.unwrap().l().unwrap();
    // println!("running !!!");
    // let mysql_value = info.get(JObject::from(mysql_key));
    // match(mysql_value) {
    //     Ok(val) => {
    //         println!("OK")
    //     }
    //     Err(err) => {
    //         println!("error !!!!!!");
    //         println!("err is {}", format!("{:?}", err));
    //     }
    // }
    // println!("after running!!");
    // let redis_value = info.get(JObject::from(redis_key)).unwrap().unwrap();
    // let mut real_map = HashMap::<String, String>::new();
    // // real_map.insert(String::from("mysql"), String::from(env.get_string(JString::from(mysql_value)).unwrap()));
    // // real_map.insert(String::from("redis"), String::from(env.get_string(JString::from(redis_value)).unwrap()));
    let mut real_map = HashMap::<String, String>::new();
    real_map.insert(String::from("mysql"), String::from("mysql://root:123@localhost:3306/stock"));
    real_map.insert(String::from("redis"), String::from("redis://127.0.0.1/"));
    real_map.insert(String::from("mysql"), String::from(env.get_string(JString::from(mysql_value)).unwrap()));
    real_map.insert(String::from("redis"), String::from(env.get_string(JString::from(redis_value)).unwrap()));
    init(real_map);
}

#[no_mangle]
pub extern "system" fn Java_com_cassiopeia_rust_RustAPI_startTimeFetch(env: JNIEnv, class: JClass) {
    let columns = vec!["ts_code"];
    let stock_codes_rows = executor::block_on(sql::query_stock_list(&columns, " where market in ('主板', '中小板')")).unwrap();
    let mut count = 0;
    let mut each_thread_codes = Vec::<String>::with_capacity(EACH_THREAD_FETCH_NUM);
    for row in &stock_codes_rows {
        let ts_code: String = row.get("ts_code");
        each_thread_codes.push(ts_code);
        count = count + 1;
        if count == EACH_THREAD_FETCH_NUM {
            println!("thread num!!!!!");
            task::spawn(fetch_index_info(each_thread_codes));
            each_thread_codes = Vec::<String>::with_capacity(EACH_THREAD_FETCH_NUM);
            count = 0;
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_cassiopeia_rust_RustAPI_calculateHistoryDownSync(env: JNIEnv, class: JClass) {
    task::block_on(calculate_history_down());
}

#[no_mangle]
pub extern "system" fn Java_com_cassiopeia_rust_RustAPI_commonSelect(env: JNIEnv, class: JClass) {
    task::spawn(async {
        let mut select = AllSelectStrategy::new().await;
        select.initialize().await;
        select.select().await;
    });
}

#[no_mangle]
pub extern "system" fn Java_com_cassiopeia_rust_RustAPI_trackSold(env: JNIEnv, class: JClass) {
    task::spawn(async {
        let mut sold = TrackSold::new();
        sold.initialize().await;
        sold.sold().await;
    });
}