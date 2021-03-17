use crate::utils::time_utils::{SleepDuringStop};
use chrono::{DateTime, Local};
use crate::sql;
use sqlx::{MySql, Row};
use sqlx::pool::PoolConnection;
use crate::selector::{CommonSelectRst, SingleCommonRst, SHORT_TYPE, SINGLE_PRICE, LONG_TYPE, FINAL_TYPE};
use crate::utils::time_utils;

/// 对于三张表做特殊处理
/// short_time_select, long_time_select, wait_select
/// 首先有一张表记录当天动态实时的数据，然后有一张历史表，历史表里面记录每天的选股历史
pub struct CommonTimeRstProcess {
    all_result: CommonSelectRst,
    time_check: SleepDuringStop,
}

impl CommonTimeRstProcess {
    pub fn new() -> Self {
        CommonTimeRstProcess {
            all_result: CommonSelectRst::new(),
            time_check: SleepDuringStop::new(),
        }
    }

    pub async fn initialize(&mut self) {
        self.refresh().await;
    }

    pub async fn refresh(&mut self) {
        self.all_result = CommonSelectRst::new();
        // 第零步：获取当前的日期
        let curr_date_str = time_utils::curr_date_str("%Y%m%d");
        let all_type = vec!["short_time_select", "long_time_select", "wait_select"];
        for item in all_type {
            let mut sql = String::from("select * from ");
            sql = sql + item + "where in_time='";
            sql = sql + curr_date_str.as_str() + "'";
            sql::async_common_query(sql.as_str(), |rows| {
                for row in rows {
                    let mut single_rst = SingleCommonRst {
                        ts_code: row.get("ts_code"),
                        ts_name: "".to_string(),
                        curr_price: row.get::<'_, f64, &str>("in_price"),
                        level: row.get::<'_, i64, &str>("level"),
                        source: row.get("source"),
                        level_pct: row.get::<'_, f64, &str>("level_pct"),
                        line_style: SINGLE_PRICE,
                        rst_style: SHORT_TYPE
                    };
                    let mut temp_rst_style = SHORT_TYPE;
                    match item {
                        "short_time_select" => {temp_rst_style = SHORT_TYPE;},
                        "long_time_select" => {temp_rst_style = LONG_TYPE;},
                        "wait_select" => {temp_rst_style = FINAL_TYPE;},
                        &_ => {}
                    }
                    single_rst.rst_style = temp_rst_style;
                    self.all_result.add_selected(single_rst);
                }
            }).await;
        }
    }

    pub async fn process(&mut self, rst: &CommonSelectRst, curr_time: &DateTime<Local>) -> Vec<String> {
        let new_stocks = self.all_result.append(rst);
        // 判定是否已经到了交易日结束的时候了
        self.sync_to_db(self.time_check.check_curr_night_rest(curr_time)).await;
        new_stocks
    }

    async fn sync_to_db(&self, is_history: bool) {
        let pool = crate::initialize::MYSQL_POOL.get().unwrap();
        let mut conn = pool.acquire().await.unwrap();
        let curr_time_str = self.all_result.ts.format("%Y%m%d %H:%M:%S").to_string();
        if !is_history {
            let mut del_sql = "delete from short_time_select";
            sql::async_common_exe(del_sql).await;
            del_sql = "delete from long_time_select";
            sql::async_common_exe(del_sql).await;
            del_sql = "delete from wait_select";
            sql::async_common_exe(del_sql).await;
        }

        for item in &self.all_result.select_rst {
            if item.rst_style & SHORT_TYPE > 0 {
                sync_to_short_time(item, &curr_time_str, is_history, &mut conn).await;
            }
            if item.rst_style & LONG_TYPE > 0 {
                sync_to_long_time(item, &curr_time_str, is_history, &mut conn).await;
            }
            if item.rst_style & FINAL_TYPE > 0 {
                sync_to_wait_select(item, &curr_time_str, is_history, &mut conn).await;
            }
        }
    }
}

async fn sync_to_short_time(single_rst: &SingleCommonRst, curr_time: &String, is_history: bool,
                            conn: &mut PoolConnection<MySql>) {
    let mut sql = String::from("insert into ");
    if is_history {
        sql = sql + "short_time_history";
    }
    else {
        sql = sql + "short_time_select";
    }
    sql = sql + "(ts_code, in_price, in_time, source, level, line_style) values(?,?,?,?,?,?)";
    let mut query = sqlx::query(sql.as_str());
    query = query.bind(single_rst.ts_code.clone());
    query = query.bind(single_rst.curr_price);
    query = query.bind(curr_time.clone());
    query = query.bind(single_rst.source.clone());
    query = query.bind(single_rst.level);
    query = query.bind(single_rst.line_style);
    match query.execute(conn).await {
        Ok(_) => {},
        Err(err) => {
            // TODO 日志记录一下
            println!("err is {}", format!("{:?}", err));
        },
    }

}

async fn sync_to_long_time(single_rst: &SingleCommonRst, curr_time: &String, is_history: bool,
                           conn: &mut PoolConnection<MySql>) {
    let mut sql = String::from("insert into ");
    if is_history {
        sql = sql + "long_time_history";
    }
    else {
        sql = sql + "long_time_select";
    }
    sql = sql + "(ts_code, in_price, in_time, source, level, line_style) values(?,?,?,?,?,?)";
    let mut query = sqlx::query(sql.as_str());
    query = query.bind(single_rst.ts_code.clone());
    query = query.bind(single_rst.curr_price);
    query = query.bind(curr_time.clone());
    query = query.bind(single_rst.source.clone());
    query = query.bind(single_rst.level);
    query = query.bind(single_rst.line_style);
    match query.execute(conn).await {
        Ok(_) => {},
        Err(err) => {
            // TODO 日志记录一下
            println!("err is {}", format!("{:?}", err));
        },
    }

}

async fn sync_to_wait_select(single_rst: &SingleCommonRst, curr_time: &String, is_history: bool,
                             conn: &mut PoolConnection<MySql>) {
    let mut sql = String::from("insert into ");
    if is_history {
        sql = sql + "wait_select_history";
    }
    else {
        sql = sql + "wait_select";
    }
    sql = sql + "(ts_code, in_price, in_date, in_reason, level, line_style) values(?,?,?,?,?,?)";
    let mut query = sqlx::query(sql.as_str());
    query = query.bind(single_rst.ts_code.clone());
    query = query.bind(single_rst.curr_price);
    query = query.bind(curr_time.clone());
    query = query.bind(single_rst.source.clone());
    query = query.bind(single_rst.level);
    query = query.bind(single_rst.line_style);
    match query.execute(conn).await {
        Ok(_) => {},
        Err(err) => {
            // TODO 日志记录一下
            println!("err is {}", format!("{:?}", err));
        },
    }

}