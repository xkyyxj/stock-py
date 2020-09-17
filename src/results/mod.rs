use sqlx::MySql;
use sqlx::query::Query;
use sqlx::mysql::MySqlArguments;

mod result;

pub use result::{ StockBaseInfo, InLow, TimeIndexInfo, TimeIndexBatchInfo };

pub trait DBResult {
    fn new() -> Self;
    fn insert(&self) -> Query<'_, MySql, MySqlArguments>;
    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments>;
}