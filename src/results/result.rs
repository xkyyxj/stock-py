use super::DBResult;
use sqlx::MySql;
use sqlx::mysql::MySqlArguments;
use sqlx::query::Query;
use chrono::{DateTime, Local};
use std::ops::Add;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::str::FromStr;

//TODO 这里面的东西可以再拆分一下，太乱了

pub struct InLow {
    pub(crate) pk_low: i32,
    pub(crate) ts_code: Option<String>,
    pub(crate) ts_name: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) in_price: f64
}

/// 最近一段时间盈利最多的股票
pub struct MaxWin {
    pub(crate) pk_maxwin: i64,                      // 主键
    pub(crate) ts_code: String,                     // 股票编码
    pub(crate) in_price: f64,                       // 进入价格
    pub(crate) start_date: String,                  // 从该天开始计算收益
    pub(crate) delta_days: i64,                     // 计算周期（in_date - start_date之间交易日）
    pub(crate) win_pct: f64,                        // 获利百分比
    pub(crate) industry: String,                    // 所属行业
    pub(crate) in_date: String                      // 计算时间
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
        query = query.bind(self.ts_code.as_ref());
        query = query.bind(self.ts_name.as_ref());
        query = query.bind(self.date.as_ref());
        query.bind(self.in_price)
    }

    fn query(where_part: Option<String>) -> Vec<Box<InLow>> {
        // TODO -- un finished
        unimplemented!()
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

    fn query(where_part: Option<String>) -> Vec<Box<StockBaseInfo>> {
        // TODO -- finish it
        unimplemented!()
    }
}

