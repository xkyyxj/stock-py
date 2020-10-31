use sqlx::MySql;
use sqlx::query::Query;
use sqlx::mysql::MySqlArguments;

mod result;
mod quick_down_then_flow;
mod history_down;
mod time_index_info;

pub use result::{ StockBaseInfo, InLow };
pub use time_index_info::{ TimeIndexInfo, TimeIndexBatchInfo };
pub use history_down::HistoryDown;

/// 数据库结果的Trait
pub trait DBResult {
    fn new() -> Self;
    fn insert(&self) -> Query<'_, MySql, MySqlArguments>;
    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments>;
    fn query(where_part: Option<String>) -> Vec<Box<Self>>;
}