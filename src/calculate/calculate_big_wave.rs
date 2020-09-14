use crate::sql;
use futures::task::SpawnExt;
use sqlx::{Row, Acquire, MySql};
use futures::channel::mpsc::{ self, Sender };
use futures::{SinkExt, StreamExt};
use sqlx::pool::PoolConnection;
use crate::results::{DBResult};
use std::collections::HashMap;
use chrono::Local;

pub async fn calculate_big_wave() -> bool {
    let columns = vec!["ts_code", "name"];
    let query_list_fut = sql::query_stock_list(&columns, "");
    let stock_list = query_list_fut.await.unwrap();
    let stock_num = stock_list.len();
    let each_group_num = stock_num / crate::config::MYSQL_MAX_CONNECTION + 1;
    println!("all stock num is {}, and each group num is {}",stock_num, each_group_num);

    // buffer的大小是4000会不会有问题？
    let (tx, rx) = mpsc::channel::<u32>(4000);
    let thread_pool = crate::THREAD_POOL.get().unwrap();

    let mut count = 0;
    let mut grp_count = 0;
    let mut ts_codes = Vec::<String>::with_capacity(each_group_num);
    let mut code2name_map = HashMap::<String, String>::with_capacity(each_group_num);
    for row in &stock_list {
        count = count + 1;
        let ts_code: String = row.get("ts_code");
        let ts_name: String = row.get("name");
        code2name_map.insert(String::from(&ts_code), ts_name);
        ts_codes.push(ts_code);
        if count == each_group_num {
            println!("group count is {}", grp_count);
            let temp_tx = mpsc::Sender::clone(&tx);
            let conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
            thread_pool.spawn(calculate_in_low_s(conn, ts_codes, temp_tx, code2name_map)).unwrap();
            grp_count = grp_count + 1;
            count = 0;
            ts_codes = Vec::<String>::with_capacity(each_group_num);
            code2name_map = HashMap::<String, String>::with_capacity(each_group_num);
        }
    }

    let mut conn = crate::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
    thread_pool.spawn(calculate_in_low_s(conn, ts_codes, tx, code2name_map)).unwrap();
    grp_count = grp_count + 1;

    // 同步机制，确保所有的计算都已经完成
    let ret_val = rx.collect::<Vec<u32>>().await;
    println!("cal finished!!");
    ret_val.len() == grp_count
}

/// 单条股票的计算
async fn calculate_in_low_s(mut conn: PoolConnection<MySql>,
                            stock_codes: Vec<String>, mut tx: Sender<u32>,
                            code2name_map: HashMap<String, String>) {
    if stock_codes.is_empty() {
        match tx.send(1).await {
            Ok(_) => {},
            Err(_) => {println!("cal success but send message failed!")}
        }
        return;
    }

    let date_time = Local::now();
    let curr_date_str = date_time.format("%Y%m%d").to_string();
    for item in stock_codes {
        let all_vos = sql::query_stock_base_info_a_with_conn(
            &mut conn,
            item.as_str(),
            " and trade_date > '20180101' order by trade_date").await;

        if all_vos.is_empty() {
            continue;
        }


        //sql::insert(&mut conn, in_low).await;
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
    match tx.send(1).await {
        Ok(_) => {},
        Err(_) => {println!("cal success but send message failed!")}
    };
}