use crate::sql;
use sqlx::{MySql, Row};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use async_std::task;
use sqlx::pool::PoolConnection;
use crate::results::{ HistoryDown, DBResult };
use std::collections::HashMap;
use chrono::Local;

/// 坚实基础理论：低值，估价偏低，巴菲特宣传的思想对吧
pub async fn calculate_history_down() -> bool {
    fn temp(conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calculate_history_down_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票计算是否是历史低值区间
/// 上榜条件：往前推min_history_down_days天，最后一天的价格比200天之内的最低价不高于min_history_down_up_pct
/// TODO -- 可以添加如下内容：
/// 1. MA金叉
/// 2.
async fn calculate_history_down_s(mut conn: PoolConnection<MySql>,
                            stock_codes: Vec<String>, mut tx: Sender<u32>,
                            _code2name_map: HashMap<String, String>) {
    let last_days = crate::initialize::CONFIG_INFO.get().unwrap().min_history_down_days;
    let min_up_pct = crate::initialize::CONFIG_INFO.get().unwrap().min_history_down_up_pct;
    for item in stock_codes {
        // 第一步：查询最近3年多的交易日线信息
        let mut all_close = Vec::<f64>::new();
        let mut sql = String::from("select close from stock_base_info where ts_code='");
        sql = sql + item.as_str() + "' order by trade_date desc limit 900";
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                all_close.push(row.get::<'_, f64, &str>("close"));
            }
        }).await;

        if all_close.is_empty() {
            continue;
        }

        // 第二步：开始计算
        // 计算逻辑：从最新交易记录向旧交易记录遍历，最后一天相比于该天涨了多少
        let mut history_down = HistoryDown::new();
        let mut delta_days = 0;
        let mut delta_pct = 0f64;
        // 最后一天的价格
        let last_day_close = all_close.get(0).unwrap();
        let mut his_down_price = *last_day_close;
        for i in 1..all_close.len() {
            let temp_close = all_close.get(i).unwrap();
            delta_pct = (*last_day_close - *temp_close) / *temp_close;
            if *temp_close < his_down_price {
                his_down_price = *temp_close;
            }
            if delta_pct < min_up_pct {
                delta_days = delta_days + 1;
            }
            else {
                break;
            }
        }
        if delta_days < last_days || delta_pct > min_up_pct {
            continue;
        }

        // 查询最后一天的交易日期
        let mut sql = String::from("select trade_date from stock_base_info where ts_code='");
        sql = sql + item.as_str() + "' order by trade_date desc limit 1";
        let mut trade_date = String::new();
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                trade_date = row.get("trade_date");
            }
        }).await;
        history_down.ts_code = item;
        history_down.his_down_price = his_down_price;
        history_down.in_price = *last_day_close;
        history_down.in_date = trade_date;
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