use async_std::task;
use chrono::{DateTime, Local};
use crate::utils::time_utils::SleepDuringStop;
use crate::selector::{ShortTimeSelectResult, SingleShortTimeSelectResult};
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Row};
use crate::utils::time_utils;
use crate::sql;
use sqlx::mysql::MySqlRow;

pub struct SingleShortTimeHistory {
    pub(crate) ts_code: String,
    pub(crate) ts_name: String,
    pub(crate) in_price: f64,
    pub(crate) level: i64,              // 评分：0-100分
    pub(crate) source: String,          // 来源系统，通过ema选定还是什么其他指标
    pub(crate) level_pct: f64,          // 得分的百分比
    pub(crate) line_style: i32,         // 分时线形态：-1 一直下降；0 经历过拐点(先下降后上升)；1 先上升后下降；2 一直上涨；3 一直一个价;4 反复波动
    pub(crate) five_win: f64,           // 五日盈利百分比
    pub(crate) seven_win: f64,          // 七日盈利百分比
}

impl From<&SingleShortTimeSelectResult> for SingleShortTimeHistory {
    fn from(source: &SingleShortTimeSelectResult) -> Self {
        SingleShortTimeHistory {
            ts_code: String::from(&source.ts_code),
            ts_name: String::from(&source.ts_code),
            in_price: source.curr_price,
            level: source.level,
            source: String::from(&source.source),
            level_pct: source.level_pct,
            line_style: source.line_style,
            five_win: 0.0,
            seven_win: 0.0
        }
    }
}

impl SingleShortTimeHistory {
    pub(crate) async fn sync_to_db(&self, curr_time: &String, conn: &mut PoolConnection<MySql>) {
        let mut query = sqlx::query("insert into short_time_history(ts_code, in_price, in_time, source, level, line_style)\
        values(?,?,?,?,?,?)");
        query = query.bind(self.ts_code.clone());
        query = query.bind(self.in_price);
        query = query.bind(curr_time.clone());
        query = query.bind(self.source.clone());
        query = query.bind(self.level);
        query = query.bind(self.line_style);
        match query.execute(conn).await {
            Ok(_) => {},
            Err(err) => {
                println!("err is {}", format!("{:?}", err));
            },
        }
    }
}

pub struct ShortTimeHistory {
    pub(crate) select_rst: Vec<SingleShortTimeHistory>,
    pub(crate) ts: DateTime<Local>,
}

impl From<&ShortTimeSelectResult> for ShortTimeHistory {
    fn from(source: &ShortTimeSelectResult) -> Self {
        let mut ret_val = ShortTimeHistory { select_rst: vec![], ts: source.ts.clone() };
        for item in &source.select_rst {
            ret_val.select_rst.push(SingleShortTimeHistory::from(item));
        }
        ret_val
    }
}

impl ShortTimeHistory {
    pub(crate) async fn sync_to_db(&self) {
        if self.select_rst.is_empty() {
            return;
        }

        let sql_client = crate::initialize::MYSQL_POOL.get().unwrap();
        let mut conn = sql_client.acquire().await.unwrap();
        for item in &self.select_rst {
            let curr_time_str= self.ts.format("%Y%m%d %H:%M:%S").to_string();
            item.sync_to_db(&curr_time_str, &mut conn).await;
        }
    }
}

/// 此程序应该在每个交易日结束的时候做一次，具有如下功效：
/// 1. 将short_select_intime表里面的数据移动到short_time_history当中去
/// 2. 对于超过交易日限制而没有发出卖出信号的股票，执行以下卖出逻辑，计算一下到期日的交易价格（以收盘价计）
pub async fn sync_short_history(curr_time: &DateTime<Local>) {
    // 第一步：判定是否已经到了交易日结束的时候了
    let time_check = SleepDuringStop::new();
    if !time_check.check_curr_night_rest(curr_time) {
        return;
    }

    // 第二步：开始将short_select_in_time表里面的数据移动到short_time_history当中去
    let short_time_rst = ShortTimeSelectResult::query_all().await;
    let short_time_history = ShortTimeHistory::from(&short_time_rst);
    short_time_history.sync_to_db().await;

    // 第三步：统计五天之内和七天之内的盈利百分比
    // 首先查询出来进入时间大于5天的股票
    let n_days_before = time_utils::curr_date_before_days_str(4, "%Y%m%d");
    let mut query = String::from("select pk_short_history, in_price, ts_code, in_time ");
    query = query + " from short_time_history where in_time <= '" + n_days_before.as_str() + "'";
    sql::async_common_query(query.as_str(), |rows| {
        for row in rows {
            // 而后调用函数做计算，并更新数据库表格
            let pk = row.get::<'_,i64, &str>("pk_short_history");
            let ts_code = row.get("ts_code");
            let in_time = row.get("in_time");
            let in_price = row.get::<'_, f64, &str>("in_price");
            // FIXME -- 此处新建了一个task，如果程序提前终止的话就跑不完了，不过似乎这个程序里面都是这种
            task::spawn(cal_single_row(pk, ts_code, in_time, in_price));
        }
    }).await;
}

async fn cal_single_row(pk: i64, ts_code: String, in_time: String, in_price: f64) {
    let mut sql = String::from("select close from stock_base_info where ts_code='");
    sql = sql + ts_code.as_str() + "' and trade_date > '" + in_time.as_str() + "'";
    sql = sql + " order by trade_date limit 20";
    let mut close_val = Vec::<f64>::new();
    sql::async_common_query(sql.as_str(), |rows| {
        for row in rows {
            close_val.push(row.get::<'_, f64, &str>("close"))
        }
    }).await;

    let mut five_win = 0f64;
    let mut seven_win = 0f64;
    let mut update_five = false;
    let mut update_seven = false;
    if close_val.len() > 4 {
        // 计算五日盈利（买入当天算第一天，第五天盈利如何）
        let target_close = close_val.get(3).unwrap();
        five_win = (target_close - in_price) / in_price;
        update_five = true;
    }

    if close_val.len() > 6 {
        // 计算五日盈利（买入当天算第一天，第五天盈利如何）
        let target_close = close_val.get(5).unwrap();
        seven_win = (target_close - in_price) / in_price;
        update_seven = true;
    }

    if !update_five && !update_seven {
        return;
    }

    sql = sql + five_win.to_string().as_str() + "'";
    if update_seven {
        sql = sql + ", seven_win='" + seven_win.to_string().as_str() + "'"
    }
    sql = sql + " where pk_short_history='" + pk.to_string().as_str() + "'";
    println!("update sql is {}", sql);
    sql::async_common_exe(sql.as_str()).await;
}