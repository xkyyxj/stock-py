use crate::results::DBResult;
use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::{MySqlArguments, MySqlRow};
use crate::sql;
use std::borrow::Borrow;
use std::ops::Deref;

pub struct OpeInfo {
    pub pk_ope: i64,
    pub ts_code: String,
    pub trade_date: String,
    pub ope_num: i64,
    pub ope_close: f64,
    pub ope_flag: String,
    pub win_mny: f64,
    pub win_pct: f64,
    pub select_type: String,
    pub pk_buy_ope: i64,
    pub buy_left_num: i64,
    pub simulate: bool,
}

pub fn process_single_row_for_ope_info(row: &MySqlRow, is_simulate: bool) -> OpeInfo {
    let mut temp_rst = OpeInfo::new();
    temp_rst.pk_ope = row.get("pk_ope");
    temp_rst.ts_code = row.get("ts_code");
    temp_rst.trade_date = row.get("trade_date");
    temp_rst.ope_num = row.get::<'_, i64, &str>("ope_num");
    temp_rst.ope_close = row.get::<'_, f64, &str>("ope_close");
    temp_rst.ope_flag = row.get("ope_flag");
    temp_rst.win_mny = row.get::<'_, f64, &str>("win_mny");
    temp_rst.win_pct = row.get::<'_, f64, &str>("win_pct");
    temp_rst.select_type = row.get("select_type");
    temp_rst.pk_buy_ope = row.get::<'_, i64, &str>("pk_buy_ope");
    temp_rst.buy_left_num = row.get::<'_, i64, &str>("buy_left_num");
    temp_rst.simulate = is_simulate;
    temp_rst
}

impl DBResult for OpeInfo {
    fn new() -> Self {
        OpeInfo {
            pk_ope: 0,
            ts_code: "".to_string(),
            trade_date: "".to_string(),
            ope_num: 0,
            ope_close: 0.0,
            ope_flag: "".to_string(),
            win_mny: 0.0,
            win_pct: 0.0,
            select_type: "".to_string(),
            pk_buy_ope: 0,
            buy_left_num: 0,
            simulate: false
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        let mut insert_sql = "";
        if self.simulate {
            insert_sql = "insert into operate_info_simulate (ts_code, trade_date, \
            ope_num, ope_close, ope_flag, win_mny, win_pct, select_type, pk_buy_ope, buy_left_num) \
            values(?,?,?,?,?,?,?,?,?,?)";
        } else {
            insert_sql = "insert into operate_info (ts_code, trade_date, \
            ope_num, ope_close, ope_flag, win_mny, win_pct, select_type, pk_buy_ope, buy_left_num) \
            values(?,?,?,?,?,?,?,?,?,?)";
        }
        sqlx::query(insert_sql)
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.trade_date);
        query = query.bind(self.ope_num);
        query = query.bind(self.ope_close);
        query = query.bind(&self.ope_flag);
        query = query.bind(self.win_mny);
        query = query.bind(self.win_pct);
        query = query.bind(&self.select_type);
        query = query.bind(self.pk_buy_ope);
        query.bind(self.buy_left_num)
    }

    fn query(query_info: &super::QueryInfo) -> Vec<Box<Self>> {
        let mut final_sql = super::process_query_info(query_info);

        let mut final_rst = Vec::<Box<OpeInfo>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row_for_ope_info(row, false)));
            }
        });
        final_rst
    }
}

pub struct WaitSold {
    pub pk_wait_sold: i64,
    pub ts_code: String,
    pub trade_date: String,
    pub ope_num: i64,
    pub ope_close: f64,
    pub win_mny: f64,
    pub win_pct: f64,
    pub select_type: String,
    pub pk_buy_ope: i64,
}

pub fn process_single_row_for_wait_sold(row: &MySqlRow) -> WaitSold {
    let mut temp_rst = WaitSold::new();
    temp_rst.pk_wait_sold = row.get("pk_wait_sold");
    temp_rst.ts_code = row.get("ts_code");
    temp_rst.trade_date = row.get("trade_date");
    temp_rst.ope_num = row.get::<'_, i64, &str>("ope_num");
    temp_rst.ope_close = row.get::<'_, f64, &str>("ope_close");
    temp_rst.win_mny = row.get::<'_, f64, &str>("win_mny");
    temp_rst.win_pct = row.get::<'_, f64, &str>("win_pct");
    temp_rst.select_type = row.get("select_type");
    temp_rst.pk_buy_ope = row.get::<'_, i64, &str>("pk_buy_ope");
    temp_rst
}

impl DBResult for WaitSold {
    fn new() -> Self {
        WaitSold {
            pk_wait_sold: 0,
            ts_code: "".to_string(),
            trade_date: "".to_string(),
            ope_num: 0,
            ope_close: 0.0,
            win_mny: 0.0,
            win_pct: 0.0,
            select_type: "".to_string(),
            pk_buy_ope: 0,
        }
    }

    fn insert(&self) -> Query<'_, MySql, MySqlArguments> {
        sqlx::query("insert into wait_sold(ts_code, trade_date, \
        ope_num, ope_close, win_mny, win_pct, select_type, pk_buy_ope) values(?,?,?,?,?,?,?,?)")
    }

    fn bind<'a>(&'a self, mut query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments> {
        query = query.bind(&self.ts_code);
        query = query.bind(&self.trade_date);
        query = query.bind(self.ope_num);
        query = query.bind(self.ope_close);
        query = query.bind(self.win_mny);
        query = query.bind(self.win_pct);
        query = query.bind(&self.select_type);
        query.bind(self.pk_buy_ope)
    }

    fn query(query_info: &super::QueryInfo) -> Vec<Box<Self>> {
        let mut final_sql = super::process_query_info(query_info);

        let mut final_rst = Vec::<Box<WaitSold>>::new();
        sql::common_query(final_sql.as_ref(), |rows| {
            for row in rows {
                final_rst.push(Box::<Self>::new(process_single_row_for_wait_sold(row)));
            }
        });
        final_rst
    }
}

