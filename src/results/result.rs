use super::DBResult;
use sqlx::MySql;
use sqlx::mysql::MySqlArguments;
use sqlx::query::Query;
use chrono::{DateTime, Local};
use std::ops::Add;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::str::FromStr;

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

    fn bind<'a>(&'a self, _query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
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

    fn bind<'a>(&'a self, _query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        unimplemented!()
    }
}

impl Display for TimeIndexBaseInfo {
    /// 返回数据格式如下：
    /// 今日开盘价,昨日收盘价,当前价格,今日最高价,今日最低价,买一,买二,买三, \
    /// 买四,买五,卖一,卖二,卖三,卖四,卖五,买一数量,买二数量,买三数量,买四数量,卖一数量,卖二数量,卖三数量,卖四数量,\
    /// 卖五数量,当前时间
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
               self.t_open, self.y_close, self.curr_price, self.t_max, self.t_min,
               self.buy_price[0], self.buy_price[1], self.buy_price[2], self.buy_price[3],
               self.buy_price[4], self.sold_price[0], self.sold_price[1], self.sold_price[2],
               self.sold_price[3], self.sold_price[4], self.buy_num[0], self.buy_num[1],
               self.buy_num[2], self.buy_num[3], self.buy_num[4], self.sold_num[0],
               self.sold_num[1], self.sold_num[2], self.sold_num[3], self.sold_num[4],
               self.curr_time)
    }
}

impl From<String> for TimeIndexBaseInfo {
    /// 字符串格式：
    /// 今日开盘价,昨日收盘价,当前价格,今日最高价,今日最低价,买一,买二,买三, \
    /// 买四,买五,卖一,卖二,卖三,卖四,卖五,买一数量,买二数量,买三数量,买四数量,卖一数量,卖二数量,卖三数量,卖四数量,\
    /// 卖五数量,当前时间
    fn from(val: String) -> Self {
        let mut ret_val = TimeIndexBaseInfo {
            t_open: 0.0,
            y_close: 0.0,
            curr_price: 0.0,
            t_max: 0.0,
            t_min: 0.0,
            buy_price: [0f64; 5],
            sold_price: [0f64; 5],
            buy_num: [0; 5],
            sold_num: [0; 5],
            curr_time: Local::now()
        };
        let v: Vec<&str> = val.split(',').collect();
        ret_val.t_open = v[0].parse().unwrap();
        ret_val.y_close = v[1].parse().unwrap();
        ret_val.curr_price  = v[2].parse().unwrap();
        ret_val.t_max = v[3].parse().unwrap();
        ret_val.t_min = v[4].parse().unwrap();
        for i in 0..5 {
            ret_val.buy_price[i] = v[5 + i].parse().unwrap();
            ret_val.sold_price[i] = v[10 + i].parse().unwrap();
            ret_val.buy_num[i] = v[15 + i].parse().unwrap();
            ret_val.sold_num[i] = v[20 + i].parse().unwrap();
        }
        ret_val.curr_time = DateTime::<Local>::from_str(v[25]).unwrap();
        ret_val
    }
}

impl From<TimeIndexInfo> for TimeIndexBaseInfo {
    fn from(val: TimeIndexInfo) -> Self {
        (&val).into()
    }
}

impl From<&TimeIndexInfo> for TimeIndexBaseInfo {
    fn from(val: &TimeIndexInfo) -> Self {
        TimeIndexBaseInfo {
            t_open: val.t_open,
            y_close: val.y_close,
            curr_price: val.curr_price,
            t_max: val.t_max,
            t_min: val.t_min,
            buy_price: val.buy_price,
            sold_price: val.sold_price,
            buy_num: val.buy_num,
            sold_num: val.sold_num,
            curr_time: val.curr_time
        }
    }
}

impl From<String> for TimeIndexBatchInfo {
    fn from(val: String) -> Self {
        let mut ret_val = TimeIndexBatchInfo {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            base_infos: vec![]
        };
        let v: Vec<&str> = val.split(':').collect();
        let mut temp_val = String::from(v[0]);
        ret_val.ts_code = temp_val;
        temp_val = String::from(v[1]);
        ret_val.ts_name = temp_val;
        for i in 2..v.len() {
            temp_val = String::from(v[i]);
            ret_val.base_infos.add(temp_val.into());
        }
        ret_val
    }
}

impl Display for TimeIndexBatchInfo {
    /// 返回的数据格式：股票编码：股票名称：今日开盘价,昨日收盘价,当前价格,今日最高价,今日最低价,买一,买二,买三, \
    /// 买四,买五,卖一,卖二,卖三,卖四,卖五,买一数量,买二数量,买三数量,买四数量,卖一数量,卖二数量,卖三数量,卖四数量,\
    /// 卖五数量,当前时间;今日开盘价,昨日收盘价,当前价格,今日最高价,今日最低价,买一,买二,买三, \
    /// 买四,买五,卖一,卖二,卖三,卖四,卖五,买一数量,买二数量,买三数量,买四数量,卖一数量,卖二数量,卖三数量,卖四数量,\
    /// 卖五数量,当前时间……
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut rst_str = String::new();
        rst_str = rst_str.add(self.ts_code.as_str());
        rst_str.push(':');
        rst_str = rst_str.add(self.ts_name.as_str());
        rst_str.push(':');
        for item in &self.base_infos {
            rst_str = rst_str.add(item.to_string().as_str());
            rst_str.push(';')
        }
        rst_str.push(':');
        f.pad(rst_str.as_str())
    }
}

impl TimeIndexBatchInfo {
    pub(crate) fn new() -> Self {
        TimeIndexBatchInfo {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            base_infos: vec![]
        }
    }

    pub(crate) fn add_single_info(&mut self, _single_info: &TimeIndexInfo) {
        if self.base_infos.is_empty() && self.ts_code.is_empty() {
            self.ts_code = String::from(&_single_info.ts_code);
            self.ts_name = String::from(&_single_info.ts_name);
        }
        self.base_infos.push(_single_info.into());
    }

}

