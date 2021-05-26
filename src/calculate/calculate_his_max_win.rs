use crate::{sql, utils};
use sqlx::{MySql, Row};
use futures::channel::mpsc::{ Sender };
use futures::{SinkExt};
use sqlx::pool::PoolConnection;
use std::collections::HashMap;
use async_std::task;
use chrono::Local;
use crate::results::{AirCastle, DBResult};

/// 计算历史上盈利超过某个百分比的股票
/// 基于如下思想：某只股票如果历史盈利百分比比较差的话，那么为什么认为它现在会给你带来更大的盈利？
/// TODO -- 此处只是考虑了收盘价，没有考虑复权问题
/// version: 0.1.0
pub async fn calculate_his_max_win() -> bool {
    fn temp(conn: PoolConnection<MySql>,
            stock_codes: Vec<String>, tx: Sender<u32>,
            code2name_map: HashMap<String, String>) {
        task::spawn(calculate_his_max_win_s(conn, stock_codes, tx, code2name_map));
    }
    super::calculate_wrapper(temp).await
}

/// 单条股票的计算
/// TODO -- not finished!!!!
pub async fn calculate_his_max_win_s(mut conn: PoolConnection<MySql>,
                                    stock_codes: Vec<String>, mut tx: Sender<u32>,
                                    _code2name_map: HashMap<String, String>) {
    // 基础配置信息
    let config = crate::initialize::CONFIG_INFO.get().unwrap();
    let date_time = Local::now();
    let _curr_date_str = date_time.format("%Y%m%d").to_string();
    let air_castle_days = (config.air_castle_up_days + 10).to_string();//多选择几天
    // 半年以前的时间，从当前天开始获取股票信息
    let half_year_before = utils::time_utils::curr_date_before_days_str(180, "%Y%m%d");
    let _query_sql = String::from(" and trade_date > '") + half_year_before.as_str() +
        "' order by trade_date";
    for item in stock_codes {
        // 开始分析进程
        // 查询ema_value里面的数据，然后看下是不是一直上涨
        // ema相比较于价格来说，会忽略掉轻微的价格下降，用EMA5吧
        let mut query_ema = String::from("select ema_5 from ema_value where ts_code='");
        query_ema = query_ema + item.as_str();
        query_ema = query_ema + "' order by trade_date desc limit " + air_castle_days.as_str();
        let mut all_ema_value = Vec::<f64>::new();
        sql::async_common_query(query_ema.as_str(), |rows| {
            for row in rows {
                all_ema_value.push(row.get::<'_, f64, &str>("ema_5"));
            }
        }).await;
        if all_ema_value.is_empty() {
            continue;
        }
        let mut up_days = 0;
        let mut pre_ema = all_ema_value[0];
        for item in all_ema_value {
            if item <= pre_ema {
                up_days = up_days + 1;
                pre_ema = item;
            }
            else {
                break;
            }
        }
        if up_days < config.air_castle_up_days {
            continue;
        }

        let mut base_info_sql = String::from("select close from stock_base_info where ts_code='");
        base_info_sql = base_info_sql + item.as_str() + "' order by trade_date desc limit ";
        base_info_sql = base_info_sql + air_castle_days.as_str();
        let mut base_infos = Vec::<f64>::new();
        sql::async_common_query(base_info_sql.as_str(), |rows| {
            for row in rows {
                base_infos.push(row.get::<'_, f64, &str>("close"));
            }
        }).await;
        // println!("sql is {}, up days is {}", base_info_sql, up_days);

        let first_index = up_days as usize - 1;
        let first_close = base_infos.get(first_index).unwrap();
        let last_close = base_infos.first().unwrap();
        let up_pct = (last_close - first_close) / first_close;
        // println!("first index is {}, first_close is {}, last close is {}", first_index, first_close, last_close);
        if up_pct >= config.air_castle_up_pct {
            // 加入到上涨空中楼阁行列当中
            let mut air_castle_val = AirCastle::new();
            air_castle_val.ts_code = String::from(item);
            air_castle_val.in_price = *last_close;
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