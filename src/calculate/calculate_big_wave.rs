use crate::sql;
use sqlx::{MySql};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use async_std::task;
use sqlx::pool::PoolConnection;
use std::collections::HashMap;
use chrono::Local;

pub async fn calculate_big_wave() -> bool {
    fn temp(conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calculate_big_wave_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票的计算
async fn calculate_big_wave_s(mut conn: PoolConnection<MySql>,
                            stock_codes: Vec<String>, mut tx: Sender<u32>,
                            _code2name_map: HashMap<String, String>) {
    let date_time = Local::now();
    let _curr_date_str = date_time.format("%Y%m%d").to_string();
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