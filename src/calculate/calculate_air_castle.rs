use crate::{sql, utils};
use sqlx::{MySql};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use sqlx::pool::PoolConnection;
use std::collections::HashMap;
use chrono::Local;
use crate::results::{AirCastle, DBResult};

/// 空中楼阁理论：疯涨的可能会继续疯涨，博傻博傻!!!!!!
pub async fn calculate_air_castle() -> bool {
    fn temp(mut conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, mut tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(calculate_air_castle_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票的计算
async fn calculate_air_castle_s(mut conn: PoolConnection<MySql>,
                              stock_codes: Vec<String>, mut tx: Sender<u32>,
                              _code2name_map: HashMap<String, String>) {
    // 基础配置信息
    let config = crate::initialize::CONFIG_INFO.get().unwrap();
    let date_time = Local::now();
    let _curr_date_str = date_time.format("%Y%m%d").to_string();
    // 半年以前的时间，从当前天开始获取股票信息
    let half_year_before = utils::time_utils::curr_date_before_days_str(180, "%Y%m%d");
    let query_sql = String::from(" and trade_date > '") + half_year_before.as_str() +
        "' order by trade_date";
    for item in stock_codes {
        let mut all_vos = sql::query_stock_base_info_a_with_conn(
            &mut conn,
            item.as_str(), query_sql.as_str()).await;

        if all_vos.len() < config.air_castle_up_days as usize {
            continue;
        }
        all_vos.reverse();

        // 开始分析进程
        let mut first_close = all_vos[0].close;
        let last_close = all_vos[0].close;
        let mut up_days = 0;
        for i in 0..all_vos.len() {
            if all_vos[i].pct_chg < 0 as f64 {
                up_days = up_days + 1;
                break;
            }
            else {
                first_close = all_vos[i].close;
            }
        }
        if up_days < config.air_castle_up_days {
            continue;
        }

        let up_pct = (last_close - first_close) / first_close;
        if up_pct >= config.air_castle_up_pct {
            // 加入到上涨空中楼阁行列当中
            let mut air_castle_val = AirCastle::new();
            air_castle_val.ts_code = String::from(item);
            air_castle_val.in_price = last_close;
            air_castle_val.in_date = String::from(&_curr_date_str);
            air_castle_val.up_days = up_days;
            air_castle_val.ave_day_up_pct = up_pct / up_days as f64;
            air_castle_val.up_pct = up_pct;
            sql::insert(&mut conn, air_castle_val).await;
        }
    }
    // 最近三个月的最低价，或者最后一天的价格比之于最低价的涨幅低于5%
    match tx.send(1).await {
        Ok(_) => { println!("cal group finished!") },
        Err(_) => { println!("cal success but send message failed!")}
    };
}
