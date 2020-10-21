use crate::sql;
use futures::task::SpawnExt;
use sqlx::{Row, MySql};
use futures::channel::mpsc::{ self, Sender };
use futures::{SinkExt, StreamExt};
use sqlx::pool::PoolConnection;
use crate::results::{ InLow, DBResult };
use std::collections::HashMap;
use chrono::Local;

pub async fn calculate_in_low() -> bool {
    fn temp(mut conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, mut tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(calculate_in_low_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票的计算
async fn calculate_in_low_s(mut conn: PoolConnection<MySql>,
                            stock_codes: Vec<String>, mut tx: Sender<u32>,
                            code2name_map: HashMap<String, String>) {
    let date_time = Local::now();
    let curr_date_str = date_time.format("%Y%m%d").to_string();
    for item in stock_codes {
        // FIXME -- 此处写死了一个日期？？？？？？？？？？
        let all_vos = sql::query_stock_base_info_a_with_conn(
            &mut conn,
            item.as_str(),
            " and trade_date > '20180101' order by trade_date").await;

        if all_vos.is_empty() {
            continue;
        }

        let mut min_close = 10000f64;
        for item in &all_vos {
            if item.close < min_close {
                min_close = item.close;
            }
        }
        let last_day_close = all_vos.get(all_vos.len() - 1).unwrap().close;
        let up_pct = (last_day_close - min_close) / min_close;
        if up_pct > 0.05 {
            continue;
        }

        let mut in_low = InLow::new();
        in_low.ts_code = Some(String::from(&item));
        if let Some(name_str) = code2name_map.get(&item) {
            in_low.ts_name = Some(String::from(name_str));
        };

        in_low.date = Some(String::from(&curr_date_str));
        in_low.in_price = all_vos[all_vos.len() - 1].close;

        sql::insert(&mut conn, in_low).await;
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
    match tx.send(1).await {
        Ok(_) => {},
        Err(_) => {println!("cal success but send message failed!")}
    };
}