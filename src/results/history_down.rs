use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use crate::results::{DBResult, Elided};
use crate::sql;
use std::ops::Add;

pub struct HistoryDown {
    pub(crate) pk_history_down: i64,        // 主键
    pub(crate) ts_code: String,             // 股票编码
    pub(crate) in_date: String,             // 进入榜单时间
    pub(crate) in_price: f64,               // 进入榜单时间的价格
    pub(crate) history_len: i64,            // 多少交易日以前的历史最低价
    pub(crate) delta_pct: f64,              // (in_price - his_down_price) / his_down_price
    pub(crate) his_down_price: f64,         // 历史最低价格
}

fn process_single_row_for_history_down(row: &MySqlRow) -> HistoryDown {
    let mut temp_rst = HistoryDown::new();
    let mut temp_str: String = row.get("ts_code");
    temp_rst.ts_code = temp_str;
    let mut temp_float: f64 = row.get("his_down_price");
    temp_rst.his_down_price = temp_float;
    temp_float = row.get("in_price");
    temp_rst.in_price = row.get("in_price");
    temp_rst.delta_pct = row.get("delta_pct");
    temp_rst.history_len = row.get("history_len");
    temp_rst.in_date = row.get("in_date");
    temp_rst.pk_history_down = row.get("pk_history_down");
    temp_rst
}

impl DBResult for HistoryDown {
    fn new() -> Self {
        HistoryDown {
            pk_history_down: 0,
            ts_code: "".to_string(),
            in_date: "".to_string(),
            in_price: 0.0,
            history_len: 0,
            delta_pct: 0.0,
            his_down_price: 0.0,
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into history_down(ts_code, in_date, \
        in_price, history_len, delta_pct, his_down_price) values(?,?,?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.in_date);
        query = query.bind(self.in_price);
        query = query.bind(self.history_len);
        query = query.bind(self.delta_pct);
        query.bind(self.his_down_price)
    }

    fn query(where_part: Option<String>) -> Vec<Box<Self>> {
        let mut final_sql = String::from("select * from history_down ");
        final_sql = super::process_where_part(final_sql, where_part);

        let mut final_rst = Vec::<Box<HistoryDown>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row_for_history_down(row)));
            }
        });
        final_rst
    }
}