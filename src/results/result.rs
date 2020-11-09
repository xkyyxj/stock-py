use super::DBResult;
use sqlx::{MySql, Row};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::query::Query;
use crate::sql;
use crate::results::Elided;

//TODO 这里面的东西可以再拆分一下，太乱了

pub struct InLow {
    pub(crate) pk_low: i32,
    pub(crate) ts_code: Option<String>,
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
    pub(crate) trade_date: String,
    pub(crate) ts_code: String,
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
            date: None,
            in_price: 0f64
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into in_low(ts_code, date, in_price) values(?,?,?)")
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
        query = query.bind(self.date.as_ref());
        query.bind(self.in_price)
    }

    fn query(where_part: Option<String>) -> Vec<Box<InLow>> {
        // TODO -- un finished
        unimplemented!()
    }
}

fn process_single_row(row: &MySqlRow) -> StockBaseInfo {
    let mut temp_rst = StockBaseInfo::new();
    let mut temp_str: String = row.get("ts_code");
    temp_rst.ts_code = temp_str;
    temp_rst.high = row.get("high");
    temp_rst.low = row.get("low");
    temp_rst.open = row.get("open");
    temp_rst.close = row.get("close");
    temp_rst.vol = row.get("vol");
    temp_rst.amount = row.get("amount");
    temp_rst.change = row.get("change");
    temp_rst.pct_chg = row.get("pct_chg");
    temp_rst.pre_close = row.get("pre_close");
    temp_rst
}

impl DBResult for StockBaseInfo {
    fn new() -> Self {
        StockBaseInfo {
            trade_date: "".to_string(),
            ts_code: "".to_string(),
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

    fn query(where_part: Option<String>) -> Vec<Box<Self>> {
        let mut final_sql = String::from("select * from history_down ");
        final_sql = super::process_where_part(final_sql, where_part);

        let mut final_rst = Vec::<Box<Self>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row(row)));
            }
        });
        final_rst
    }

}

