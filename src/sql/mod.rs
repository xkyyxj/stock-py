use std::ops::Add;
use sqlx::{Row, Error, MySql};
use sqlx::mysql::MySqlRow;
use futures::executor;
use crate::results::{StockBaseInfo, DBResult};
use sqlx::pool::PoolConnection;

/// 通用查询逻辑
pub fn common_query(sql: &str, mut f: impl FnMut(&Vec<MySqlRow>)) {
    let pool = crate::MYSQL_POOL.get().unwrap();
    let query_fut = sqlx::query(sql).fetch_all(pool);
    let all_rows = executor::block_on(query_fut).unwrap();
    f(&all_rows);
}

pub async fn insert(conn: &mut PoolConnection<MySql>, val: impl DBResult) -> bool {
    let mut query = val.insert();
    query = val.bind(query);
    match query.execute(conn).await {
        Ok(_) => true,
        Err(err) => {
            println!("err is {}", format!("{:?}", err));
            false
        },
    }
}

pub fn query_all_stock_base_info() {

}

/// 查询所有的股票列表
/// #columns : 查询的列
/// #where_part : 过滤条件，需自带“where”，笑
pub async fn query_stock_list(columns: &Vec<&str>, where_part: &str) -> Result<Vec<MySqlRow>, Error> {
    let mut query_sql = String::new();
    query_sql = query_sql.add("select ");
    if columns.is_empty() {
        query_sql = query_sql.add("* from stock_list ");
    }
    else {
        for item in columns {
            query_sql = query_sql.add(item).add(",");
        }
        // 弹出最后一个","
        query_sql.pop();
        query_sql = query_sql.add(" from stock_list ");
    }

    if !where_part.is_empty() {
        query_sql = query_sql.add(where_part);
    }

    let conn = super::MYSQL_POOL.get().unwrap();
    sqlx::query(query_sql.as_str()).fetch_all(conn).await
}

pub async fn query_stock_base_info(stock_code: &str, where_part: &str) -> Result<Vec<MySqlRow>, Error> {
    let mut query_sql = String::new();
    query_sql = query_sql.add("select * from stock_base_info where ts_code='");
    query_sql = query_sql.add(stock_code).add("'");
    if !where_part.is_empty() {
        query_sql = query_sql.add(where_part);
    }
    let conn = super::MYSQL_POOL.get().unwrap();
    sqlx::query(query_sql.as_str()).fetch_all(conn).await
}

pub async fn query_stock_base_info_with_conn(conn: &mut PoolConnection<MySql>, stock_code: &str, where_part: &str) -> Result<Vec<MySqlRow>, Error> {
    let mut query_sql = String::new();
    query_sql = query_sql.add("select * from stock_base_info where ts_code='");
    query_sql = query_sql.add(stock_code).add("'");
    if !where_part.is_empty() {
        query_sql = query_sql.add(where_part);
    }
    sqlx::query(query_sql.as_str()).fetch_all(conn).await
}

pub async fn query_stock_base_info_a(stock_code: &str, where_part: &str) -> Vec::<StockBaseInfo> {
    let all_rows = query_stock_base_info(stock_code, where_part).await.unwrap();

    let mut ret_vos = Vec::<StockBaseInfo>::with_capacity(all_rows.len());
    for row in all_rows {
        let mut temp_val = StockBaseInfo::new();
        let ts_code: String = row.get("ts_code");
        temp_val.trade_date = Some(ts_code);
        let trade_date: String = row.get("trade_date");
        temp_val.trade_date = Some(trade_date);
        let mut temp_f: f64 = row.get("close");
        temp_val.close = temp_f;
        temp_f = row.get("open");
        temp_val.open = temp_f;
        temp_f = row.get("high");
        temp_val.high = temp_f;
        temp_f = row.get("low");
        temp_val.low = temp_f;
        temp_f = row.get("amount");
        temp_val.amount = temp_f;
        temp_f = row.get("pre_close");
        temp_val.pre_close = temp_f;
        temp_f = row.get("change");
        temp_val.change = temp_f;
        temp_f = row.get("pct_chg");
        temp_val.pct_chg = temp_f;

        ret_vos.push(temp_val);
    }

    ret_vos
}

pub async fn query_stock_base_info_a_with_conn(conn: &mut PoolConnection<MySql>, stock_code: &str, where_part: &str) -> Vec::<StockBaseInfo> {
    let all_rows = query_stock_base_info_with_conn(conn, stock_code, where_part).await.unwrap();

    let mut ret_vos = Vec::<StockBaseInfo>::with_capacity(all_rows.len());
    for row in all_rows {
        let mut temp_val = StockBaseInfo::new();
        let ts_code: String = row.get("ts_code");
        temp_val.trade_date = Some(ts_code);
        let trade_date: String = row.get("trade_date");
        temp_val.trade_date = Some(trade_date);
        let mut temp_f: f64 = row.get("close");
        temp_val.close = temp_f;
        temp_f = row.get("open");
        temp_val.open = temp_f;
        temp_f = row.get("high");
        temp_val.high = temp_f;
        temp_f = row.get("low");
        temp_val.low = temp_f;
        temp_f = row.get("amount");
        temp_val.amount = temp_f;
        temp_f = row.get("pre_close");
        temp_val.pre_close = temp_f;
        temp_f = row.get("change");
        temp_val.change = temp_f;
        temp_f = row.get("pct_chg");
        temp_val.pct_chg = temp_f;

        ret_vos.push(temp_val);
    }

    ret_vos
}