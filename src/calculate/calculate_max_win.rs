use sqlx::pool::PoolConnection;
use sqlx::MySql;
use futures::channel::mpsc::Sender;
use std::collections::HashMap;
use chrono::Local;
use crate::sql;
use crate::utils::time_utils;

/// 纯粹好奇一下，最近一段时间内获利最多的股票都有谁
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
                              stock_codes: Vec<String>, tx: Sender<u32>,
                              code2name_map: HashMap<String, String>) {
    let curr_date_str = time_utils::curr_date_str("%Y%m%d");
    let date_time = Local::now();
    for item in stock_codes {
        let all_vos = sql::query_stock_base_info_a_with_conn(
            &mut conn,
            item.as_str(),
            " and trade_date > '20180101' order by trade_date").await;

        if all_vos.is_empty() {
            continue;
        }

        // 起始计算日期
        let mut start_index = 0;
        if all_vos.len() > crate::config::MAX_WIN_CAL_PERIOD {
            start_index = all_vos.len() - crate::config::MAX_WIN_CAL_PERIOD;
        }
        // 计算周期长度（只计算交易日）
        let cal_period_num = all_vos.len() - start_index;
        let last_close = all_vos[all_vos.len() - 1].close;
        let start_close = all_vos[start_index].close;
        let win_pct = (last_close - start_close) / start_close;

    }
}