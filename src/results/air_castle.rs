use crate::results::{DBResult, Elided};
use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::MySqlArguments;
use std::ops::Add;
use crate::sql;

/// 空中楼阁理论股票
pub struct AirCastle {
    pub(crate) pk_air_castle: i64,
    pub(crate) ts_code: String,
    pub(crate) in_date: String,
    pub(crate) in_price: f64,
    pub(crate) up_days: i64,
    pub(crate) up_pct: f64,
    pub(crate) ave_day_up_pct: f64,

}

impl DBResult for AirCastle {
    fn new() -> Self {
        AirCastle {
            pk_air_castle: 0,
            ts_code: "".to_string(),
            in_date: "".to_string(),
            in_price: 0.0,
            up_days: 0,
            up_pct: 0.0,
            ave_day_up_pct: 0.0
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into air_castle(ts_code, in_date, \
        in_price, up_days, up_pct, ave_day_up_pct) values(?,?,?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.in_date);
        query = query.bind(self.in_price);
        query = query.bind(&self.up_days);
        query = query.bind(&self.up_pct);
        query.bind(&self.ave_day_up_pct)
    }

    fn query(where_part: Option<String>) -> Vec<Box<Self>> {
        let mut final_sql = String::from("select * from air_castle ");
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
                let mut temp_rst = AirCastle::new();
                let mut temp_str: String = row.get("ts_code");
                temp_rst.pk_air_castle = row.get("pk_air_castle");
                temp_rst.ts_code = temp_str;
                temp_rst.in_price = row.get("in_price");
                temp_rst.in_date = row.get("in_date");
                temp_rst.up_pct= row.get("up_pct");
                temp_rst.up_days= row.get("up_days");
                temp_rst.ave_day_up_pct= row.get("ave_day_up_pct");
                final_rst.push(Box::<Self>::new(temp_rst));
            }
        });
        final_rst
    }
}