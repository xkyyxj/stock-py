use sqlx::MySql;
use sqlx::query::Query;
use sqlx::mysql::MySqlArguments;

mod result;
mod quick_down_then_flow;
mod history_down;
mod time_index_info;
mod wait_select;
mod box_style;
mod air_castle;
mod ope_info;
mod ema_value;
mod curr_hold;

pub use result::{ StockBaseInfo, InLow };
pub use time_index_info::{ TimeIndexInfo, TimeIndexBaseInfo, TimeIndexBatchInfo };
pub use ope_info::{OpeInfo, WaitSold};
pub use history_down::HistoryDown;
pub use box_style::BoxStyle;
pub use wait_select::WaitSelect;
pub use air_castle::AirCastle;
use std::collections::HashMap;

type Elided<'a> = &'a usize;

/// 通用结果查询条件类
#[derive(Default)]
pub struct QueryInfo {
    pub where_part: Option<String>,
    pub table_name: Option<String>,
    pub columns: Vec<String>,
    pub params: HashMap<String, String>
}

/// 数据库结果的Trait
pub trait DBResult {
    fn new() -> Self;
    fn insert(&self) -> Query<'_, MySql, MySqlArguments>;
    fn bind<'a>(&'a self, query: Query<'a, MySql, MySqlArguments>) -> Query<'a, MySql, MySqlArguments>;
    fn query(query_info: &QueryInfo) -> Vec<Box<Self>>;
}

fn process_query_info(query_info: &QueryInfo) -> String {
    let mut sql = process_table_and_columns(query_info);
    process_where_part(sql, &query_info.where_part)
}

fn process_table_and_columns(query_info: &QueryInfo) -> String {
    let mut select_part = String::from("select ");
    let columns = &query_info.columns;
    if columns.is_empty() {
        select_part += " * from ";
    } else {
        for item in &query_info.columns {
            select_part += item.as_str();
        }
    }

    select_part += &query_info.table_name.as_ref().unwrap().as_str();
    select_part
}

fn process_where_part(mut final_sql: String, where_part: &Option<String>) -> String {
    if let Some(val) = where_part {
        if val.starts_with("where") {
            final_sql = final_sql + " " + val.as_str();
        } else {
            final_sql = final_sql + " where ";
            final_sql = final_sql + val.as_str();
        }
    }
    final_sql
}