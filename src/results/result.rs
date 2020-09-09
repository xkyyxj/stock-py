use super::DBResult;
use sqlx::MySql;
use sqlx::mysql::MySqlArguments;
use sqlx::query::Query;
use chrono::{DateTime, Local};

pub struct InLow {
    pub(crate) pk_low: i32,
    pub(crate) ts_code: Option<String>,
    pub(crate) ts_name: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) in_price: f64
}

pub struct TimeIndexBaseInfo {
    pub(crate) t_open: f64,                         // 今日开盘价
    pub(crate) y_close: f64,                        // 昨日收盘价
    pub(crate) curr_price: f64,                     // 当前价格
    pub(crate) t_max: f64,                          // 今日最高价
    pub(crate) t_min: f64,                          // 今日最低价
    pub(crate) buy_price: [f64; 5],                 // 一到五的买方价格，0为买一，以此类推
    pub(crate) sold_price: [f64; 5],                // 一到五的卖方价格，0为卖一，以此类推
    pub(crate) buy_num: [u64; 5],                   // 一到五的买方数量，0为买一数量，以此类推
    pub(crate) sold_num: [u64; 5],                  // 一到五的卖方数量，0为卖一数量，以此类推
    pub(crate) curr_time: DateTime<Local>           // 当前时间
}

pub struct TimeIndexBatchInfo {
    pub(crate) ts_code: String,                     // 股票名称
    pub(crate) ts_name: String,                     // 股票编码
    pub(crate) base_infos: Vec<TimeIndexBaseInfo>   // 基本信息合集
}

/// 单条分时数据
pub struct TimeIndexInfo {
    pub(crate) ts_code: String,                     // 股票名称
    pub(crate) ts_name: String,                     // 股票编码
    pub(crate) t_open: f64,                         // 今日开盘价
    pub(crate) y_close: f64,                        // 昨日收盘价
    pub(crate) curr_price: f64,                     // 当前价格
    pub(crate) t_max: f64,                          // 今日最高价
    pub(crate) t_min: f64,                          // 今日最低价
    pub(crate) deal_num: u64,                       // 成交数量
    pub(crate) deal_mny: f64,                       // 成交金额
    pub(crate) buy_price: [f64; 5],                 // 一到五的买方价格，0为买一，以此类推
    pub(crate) sold_price: [f64; 5],                // 一到五的卖方价格，0为卖一，以此类推
    pub(crate) buy_num: [u64; 5],                   // 一到五的买方数量，0为买一数量，以此类推
    pub(crate) sold_num: [u64; 5],                  // 一到五的卖方数量，0为卖一数量，以此类推
    pub(crate) curr_time: DateTime<Local>           // 当前时间
}

pub struct StockBaseInfo {
    pub(crate) trade_date: Option<String>,
    pub(crate) ts_code: Option<String>,
    pub(crate) open: f64,
    pub(crate) close: f64,
    pub(crate) high: f64,
    pub(crate) low: f64,
    pub(crate) vol: f64,
    pub(crate) amount: f64,
    pub(crate) pre_close: f64,
    pub(crate) change: f64,
    pub(crate) pct_chg: f64
}

impl DBResult for InLow {
    fn new() -> Self {
        InLow {
            pk_low: 0,
            ts_code: None,
            ts_name: None,
            date: None,
            in_price: 0f64
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into in_low(ts_code, ts_name, date, in_price) values(?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        // let t_str1 = &self.ts_code.unwrap();
        // let mut t_str = String::from(&self.ts_code.unwrap());
        // query = query.bind(t_str);
        // t_str = String::from(&self.ts_name.unwrap());
        // query = query.bind(t_str);
        // t_str = String::from(&self.date.unwrap());
        // query.bind(t_str)
        query = query.bind(self.ts_name.as_ref());
        query = query.bind(self.ts_code.as_ref());
        query = query.bind(self.date.as_ref());
        query.bind(self.in_price)
    }
}

impl DBResult for StockBaseInfo {
    fn new() -> Self {
        StockBaseInfo {
            trade_date: None,
            ts_code: None,
            open: 0.0,
            close: 0.0,
            high: 0.0,
            low: 0.0,
            vol: 0.0,
            amount: 0.0,
            pre_close: 0.0,
            change: 0.0,
            pct_chg: 0.0
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into ")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        unimplemented!()
    }
}

impl DBResult for TimeIndexInfo {
    fn new() -> Self {
        TimeIndexInfo {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            t_open: 0.0,
            y_close: 0.0,
            curr_price: 0.0,
            t_max: 0.0,
            t_min: 0.0,
            deal_num: 0,
            deal_mny: 0f64,
            buy_price: [0f64; 5],
            sold_price: [0f64; 5],
            buy_num: [0; 5],
            sold_num: [0; 5],
            curr_time: Local::now()
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        unimplemented!()
    }

    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        unimplemented!()
    }
}

impl TimeIndexBatchInfo {
    fn add_single_info(&mut self, single_info: &TimeIndexInfo) {

    }
}

