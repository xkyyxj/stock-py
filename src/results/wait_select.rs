use crate::results::{DBResult, Elided};
use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::MySqlArguments;
use std::ops::Add;
use crate::sql;

/// 备选股票
pub struct WaitSelect {
    pub(crate) pk_wait_select: i64,
    pub(crate) ts_code: String,
    pub(crate) in_date: String,
    pub(crate) in_price: f64,
    pub(crate) in_reason: String,
    pub(crate) level: i64,                  // 买入评级：0-10级，0级最低，10级最高
}

impl DBResult for WaitSelect {
    fn new() -> Self {
        WaitSelect {
            pk_wait_select: 0,
            ts_code: "".to_string(),
            in_date: "".to_string(),
            in_price: 0.0,
            in_reason: "".to_string(),
            level: 0
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into wait_select(ts_code, in_date, \
        in_price, in_reason, level) values(?,?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.in_date);
        query = query.bind(self.in_price);
        query = query.bind(&self.in_reason);
        query.bind(&self.level)
    }

    fn query(where_part: Option<String>) -> Vec<Box<Self>> {
        let mut final_sql = String::from("select * from wait_select ");
        if let Some(val) = where_part {
            if val.contains("where") {
                final_sql = final_sql.add(val.as_str());
            } else {
                final_sql = final_sql.add("where ");
                final_sql = final_sql.add(val.as_str());
            }
        }

        let mut final_rst = Vec::<Box<Self>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                let mut temp_rst = WaitSelect::new();
                let mut temp_str: String = row.get("ts_code");
                temp_rst.ts_code = temp_str;
                temp_str = row.get("in_reason");
                temp_rst.in_reason = temp_str;
                temp_rst.in_price = row.get("in_price");
                temp_rst.in_date = row.get("in_date");
                temp_rst.pk_wait_select = row.get("pk_wait_select");
                temp_rst.level = row.get("level");
                final_rst.push(Box::<Self>::new(temp_rst));
            }
        });
        final_rst
    }
}