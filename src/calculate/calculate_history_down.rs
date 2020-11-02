use crate::sql;
use sqlx::{MySql};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use sqlx::pool::PoolConnection;
use crate::results::{ HistoryDown, DBResult };
use std::collections::HashMap;
use chrono::Local;

pub async fn calculate_history_down() -> bool {
    fn temp(mut conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, mut tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(calculate_history_down_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票计算是否是历史低值区间
/// 上榜条件：
async fn calculate_history_down_s(mut conn: PoolConnection<MySql>,
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

        let mut history_down = HistoryDown::new();
        let mut delta_days = 0;
        let mut his_down_price = 100000f64;
        let mut delta_pct = 0f64;
        // 最后一天的价格
        let last_day_close = all_vos[all_vos.len() - 1].close;
        for i in 0..all_vos.len() {
            let temp_close = all_vos[all_vos.len() - i - 1].close;
            delta_pct = (last_day_close - temp_close) / temp_close;
            if delta_pct < crate::config::MIN_HISTORY_DOWN_UP_PCT {
                delta_days = delta_days + 1;
                his_down_price = temp_close;
            }
            else {
                break;
            }
        }
        if delta_days < crate::config::MIN_HISTORY_DOWN_DAYS {
            continue;
        }
        history_down.ts_code = item;
        history_down.his_down_price = his_down_price;
        history_down.in_price = last_day_close;
        history_down.in_date = String::from(all_vos[all_vos.len() - 1].trade_date.as_ref().unwrap());
        history_down.history_len = delta_days as i64;
        history_down.delta_pct = (history_down.in_price - history_down.his_down_price) /
            history_down.his_down_price;

        sql::insert(&mut conn, history_down).await;
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
    match tx.send(1).await {
        Ok(_) => {
            println!("cal group finished");
            tx.flush().await;
            tx.close().await;
        },
        Err(_) => {
            println!("cal success but send message failed!");
            tx.flush().await;
            tx.close().await;
        }
    };
}