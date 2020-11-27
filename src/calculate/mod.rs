mod calculate_big_wave;
mod calculate_max_win;
mod calculate_history_down;
mod calculate_air_castle;
mod calculate_down_then_flow;
mod calculate_quick_down;
mod win_pct_calculate;

pub use calculate_max_win::calculate_max_win;
pub use calculate_history_down::calculate_history_down;
pub use calculate_air_castle::{calculate_air_castle, calculate_air_castle_s};
pub use win_pct_calculate::win_calculate;
use crate::sql;
use std::collections::HashMap;
use futures::channel::mpsc;
use sqlx::{Row, MySql};
use futures::{StreamExt, SinkExt};
use sqlx::pool::PoolConnection;
use futures::channel::mpsc::Sender;
use combine::lib::collections::hash_map::RandomState;

// temp ----------------------------------
pub use win_pct_calculate::parse_table_info;

pub async fn calculate_wrapper(target_function: fn(PoolConnection<MySql>, Vec<String>, Sender<u32>, HashMap<String, String>)) -> bool {
    let columns = vec!["ts_code", "name"];
    let query_list_fut = sql::query_stock_list(&columns, "");
    let stock_list = query_list_fut.await.unwrap();
    let stock_num = stock_list.len();
    let each_group_num = stock_num / crate::config::MYSQL_MAX_CONNECTION + 1;
    println!("all stock num is {}, and each group num is {}",stock_num, each_group_num);

    // buffer的大小是4000会不会有问题？
    let (mut tx, rx) = mpsc::channel::<u32>(4000);

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
            let conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
            let temp_tx = mpsc::Sender::clone(&tx);
            target_function(conn, ts_codes, temp_tx, code2name_map);
            grp_count = grp_count + 1;
            count = 0;
            ts_codes = Vec::<String>::with_capacity(each_group_num);
            code2name_map = HashMap::<String, String>::with_capacity(each_group_num);
        }
    }

    let conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
    if !ts_codes.is_empty() {
        target_function(conn, ts_codes, tx, code2name_map);
    }
    else {
        tx.send(1);
        tx.flush().await.ok();
        tx.close().await.ok();
    }
    grp_count = grp_count + 1;

    // 同步机制，确保所有的计算都已经完成
    let ret_val = rx.collect::<Vec<u32>>().await;
    println!("cal finished!!");
    ret_val.len() == grp_count
}