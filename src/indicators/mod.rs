use sqlx::pool::PoolConnection;
use sqlx::MySql;
use crate::sql;

mod ema;

///
pub async fn ema_calculate_s_with_conn(length: i32, ts_code: String, mut conn: PoolConnection<MySql>,) {
    let base_infos = sql::query_stock_base_info_a_with_conn(
        &mut conn, ts_code.as_str(),"").await;
    for item in base_infos {

    }
}