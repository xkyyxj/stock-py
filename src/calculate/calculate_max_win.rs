use sqlx::pool::PoolConnection;
use sqlx::MySql;
use futures::channel::mpsc::Sender;
use std::collections::HashMap;
use futures::future::BoxFuture;
use futures::SinkExt;
use chrono::Local;
use crate::sql;

pub async fn calculate_max_win() -> bool {
    fn temp(mut conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, mut tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(calculate_max_win_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

pub async fn calculate_max_win_s(mut conn: PoolConnection<MySql>,
                              stock_codes: Vec<String>, mut tx: Sender<u32>,
                              code2name_map: HashMap<String, String>) {
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
    }
}