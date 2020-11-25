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
pub async fn calculate_air_castle() -> bool {
    fn temp(mut conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, mut tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calculate_air_castle_s(conn, stock_codes, tx, code2name_map));
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
    let air_castle_begin_date = time_utils::curr_date_before_days_str(config.air_castle_up_days, "%Y%m%d");
    // 半年以前的时间，从当前天开始获取股票信息
    let half_year_before = utils::time_utils::curr_date_before_days_str(180, "%Y%m%d");
    let query_sql = String::from(" and trade_date > '") + half_year_before.as_str() +
        "' order by trade_date";
    for item in stock_codes {
        // 开始分析进程
        // 查询ema_value里面的数据，然后看下是不是一直上涨
        // ema相比较于价格来说，会忽略掉轻微的价格下降，用EMA5吧
        let mut query_ema = String::from("select ema5 from ema_value where ts_code='");
        query_ema = query_ema + item.as_str() + "' and trade_date > '" + air_castle_begin_date.as_str();
        query_ema = query_ema + "' order by trade_date desc";
        let mut all_ema_value = Vec::<f64>::new();
        sql::async_common_query(query_ema.as_str(), |rows| {
            for row in rows {
                all_ema_value.push(row.get::<'_, f64, &str>("ema5"));
            }
        }).await;
        if all_ema_value.is_empty() {
            continue;
        }
        let mut up_days = 0;
        let mut pre_ema = all_ema_value[0];
        for item in all_ema_value {
            if item >= pre_ema {
                up_days = up_days + 1;
            }
            else {
                // 相当于放弃了这只股票（没有资格进入空中楼阁当中）
                up_days = 0;
                break;
            }
        }
        if up_days < config.air_castle_up_days {
            continue;
        }

        let mut first_close = 0f64;
        let mut last_close = 0f64;
        let mut last_close_sql = String::from("select close from stock_base_info where ts_code='");
        last_close_sql = last_close_sql + item.as_str() + "’ order by trade_date desc limit 1";
        sql::async_common_query(last_close_sql.as_str(), |rows| {
            if rows.len() > 0 {
                last_close = rows[0].get::<'_, f64, &str>("close");
            }
        }).await;

        let mut first_close_sql = String::from("select close from stock_base_info where ts_code='");
        first_close_sql = first_close_sql + item.as_str() + "' and trade_date='" + air_castle_begin_date.as_str() + "'";
        sql::async_common_query(first_close_sql.as_str(), |rows| {
            if rows.len() > 0 {
                first_close = rows[0].get::<'_, f64, &str>("close");
            }
        }).await;

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

fn query_f64_val(sql: String) {

}
