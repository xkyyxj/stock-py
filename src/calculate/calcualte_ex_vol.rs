use crate::sql;
use sqlx::{MySql, Row};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use async_std::task;
use sqlx::pool::PoolConnection;
use crate::results::{ HistoryDown, DBResult };
use std::collections::HashMap;

/// 计算异常成交量，主要是针对成交量异常升高的情况
/// 计算逻辑，4天的成交量移动均线，看移动均线是否正常
pub async fn calculate_ex_vol() -> bool {
    fn temp(conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calcualte_val_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

pub async fn calcualte_val_s(mut conn: PoolConnection<MySql>,
                             stock_codes: Vec<String>, mut tx: Sender<u32>,
                             _code2name_map: HashMap<String, String>) {


}