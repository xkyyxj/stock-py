use crate::{sql, utils};
use sqlx::{MySql, Row};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use sqlx::pool::PoolConnection;
use std::collections::HashMap;
use async_std::task;
use chrono::Local;
use crate::results::{AirCastle, DBResult};
use crate::utils::time_utils;

/// 空中楼阁理论：疯涨的可能会继续疯涨，博傻博傻!!!!!!
pub async fn calculate_down_then_flow() -> bool {
    fn temp(conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calculate_down_then_flow_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票的计算
pub async fn calculate_down_then_flow_s(mut conn: PoolConnection<MySql>,
                                        stock_codes: Vec<String>, mut tx: Sender<u32>,
                                        _code2name_map: HashMap<String, String>) {
    // 基础配置信息
    let config = crate::initialize::CONFIG_INFO.get().unwrap();
    let days = config.down_then_flow_min_days;
    for item in stock_codes {
        // 开始分析进程
        // 第一步：从history_down当中查询出所有N天以前入选的股票
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
    match tx.send(1).await {
        Ok(_) => { println!("cal group finished!") },
        Err(_) => { println!("cal success but send message failed!")}
    };
}
