use sqlx::MySql;
use sqlx::query::Query;
use sqlx::mysql::MySqlArguments;

mod result;
mod quick_down_then_flow;
mod history_down;
mod time_index_info;
mod wait_select;
mod box_style;

pub use result::{ StockBaseInfo, InLow };
pub use time_index_info::{ TimeIndexInfo, TimeIndexBatchInfo };
pub use history_down::HistoryDown;
pub use box_style::BoxStyle;
pub use wait_select::WaitSelect;

type Elided<'a> = &'a usize;

/// 数据库结果的Trait
pub trait DBResult {
    fn new() -> Self;
    fn insert(&self) -> Query<'_, MySql, MySqlArguments>;
    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments>;
    fn query(where_part: Option<String>) -> Vec<Box<Self>>;
}

fn process_where_part(mut final_sql: String, where_part: Option<String>) -> String {
    if let Some(val) = where_part {
        if val.contains("where") {
            final_sql = final_sql + " " + val.as_str();
        } else {
            final_sql = final_sql + " where ";
            final_sql = final_sql + val.as_str();
        }
    }
    final_sql
}