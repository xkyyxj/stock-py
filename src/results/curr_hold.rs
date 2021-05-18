use sqlx::mysql::{MySqlRow, MySqlArguments};
use sqlx::{Row, MySql};
use crate::results::{DBResult, QueryInfo};
use sqlx::query::Query;
use crate::sql;

pub struct CurrHold {
    pub pk_curr_hold: i64,
    pub ts_code: String,
    pub in_date: String,
    pub hold_num: i64,
    pub in_price: f64,
    pub out_price: f64,
    pub hold_days: i64,
    pub simulate: bool,
}

pub fn process_single_row_for_curr_hold(row: &MySqlRow, is_simulate: bool) -> CurrHold {
    let mut temp_rst = CurrHold::new();
    temp_rst.pk_curr_hold = row.get::<'_, i64, &str>("pk_curr_hold");
    temp_rst.ts_code = row.get("ts_code");
    temp_rst.in_date = row.get("in_date");
    temp_rst.hold_num = row.get::<'_, i64, &str>("hold_num");
    temp_rst.in_price = row.get::<'_, f64, &str>("in_price");
    temp_rst.out_price = row.get::<'_, f64, &str>("out_price");
    temp_rst.hold_days = row.get::<'_, i64, &str>("hold_days");
    temp_rst.simulate = is_simulate;
    temp_rst
}

impl DBResult for CurrHold {
    fn new() -> Self {
        CurrHold {
            pk_curr_hold: 0,
            ts_code: "".to_string(),
            in_date: "".to_string(),
            hold_num: 0,
            in_price: 0.0,
            out_price: 0.0,
            hold_days: 0,
            simulate: false
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        let mut insert_sql = "";
        if self.simulate {
            insert_sql = "insert into curr_hold_simulate (ts_code, in_date, \
            hold_num, in_price, out_price, hold_days) \
            values(?,?,?,?,?,?)";
        } else {
            insert_sql = "insert into curr_hold (ts_code, in_date, \
            hold_num, in_price, out_price, hold_days) \
            values(?,?,?,?,?,?)";
        }
        sqlx::query(insert_sql)
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.in_date);
        query = query.bind(self.hold_num);
        query = query.bind(self.in_price);
        query = query.bind(&self.out_price);
        query.bind(self.hold_days)
    }

    fn query(query_info: &QueryInfo) -> Vec<Box<Self>> {
        let mut final_sql = super::process_query_info(query_info);

        let mut final_rst = Vec::<Box<CurrHold>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row_for_curr_hold(row, false)));
            }
        });
        final_rst
    }
}