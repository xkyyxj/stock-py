use crate::results::TimeIndexBaseInfo;
use futures::{Future};
use crate::utils::time_utils::SleepDuringStop;
use chrono::{DateTime, Local, Duration};
use std::sync::mpsc;
use async_std::task::sleep;
use crate::results::DBResult;
use sqlx::query::Query;
use sqlx::{MySql};
use sqlx::mysql::MySqlArguments;
use sqlx::pool::PoolConnection;


//------------------长期筛选结果集--------------------------------------------------------------------
pub(crate) struct SingleLongTimeSelectResult {
    pub(crate) ts_code: String,
    pub(crate) ts_name: String,
    pub(crate) curr_price: f64,
    pub(crate) level: i64,              // 评分：0-100分
    pub(crate) source: String,          // 来源系统，通过ema选定还是什么其他指标
    pub(crate) level_pct: f64,          // 得分的百分比
    pub(crate) line_style: i32,         // 分时线形态：-1 一直下降；0 经历过拐点(先下降后上升)；1 先上升后下降；2 一直上涨；3 一直一个价;4 反复波动
}

impl SingleLongTimeSelectResult {
    pub(crate) fn new() -> Self {
        SingleLongTimeSelectResult {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            curr_price: 0.0,
            level: 0,
            source: "".to_string(),
            level_pct: 0.0,
            line_style: 3
        }
    }

    pub(crate) async fn insert_to_db(&self, curr_time: &String, conn: &mut PoolConnection<MySql>) {
        let mut query = sqlx::query("insert into short_select_in_time(ts_code, in_price, in_time, source, level)\
        values(?,?,?,?,?)");
        query = query.bind(self.ts_code.clone());
        query = query.bind(self.curr_price);
        query = query.bind(curr_time.clone());
        query = query.bind(self.source.clone());
        query = query.bind(self.level);
        match query.execute(conn).await {
            Ok(_) => {},
            Err(err) => {
                println!("err is {}", format!("{:?}", err));
            },
        }
    }
}

impl Clone for SingleLongTimeSelectResult {
    fn clone(&self) -> Self {
        SingleLongTimeSelectResult {
            ts_code: String::from(&self.ts_code),
            ts_name: String::from(&self.ts_name),
            curr_price: self.curr_price,
            level: self.level,
            source: String::from(&self.source),
            level_pct: self.level_pct,
            line_style: self.line_style
        }
    }
}

pub(crate) struct LongTimeSelectResult {
    pub(crate) select_rst: Vec<SingleLongTimeSelectResult>,
    pub(crate) ts: DateTime<Local>,
}

impl LongTimeSelectResult {

    pub(crate) fn new() -> Self {
        LongTimeSelectResult { select_rst: vec![], ts: Local::now() }
    }

    pub(crate) fn add_selected(&mut self, info : SingleLongTimeSelectResult) {
        self.select_rst.push(info);
    }

    /// 合并结果用于多个不同的选择策略的合并，蒋选择结果合并到最终结果当中需要用到append方法
    /// 两个结果的合并，重复的结果得分的简单相加，只在一方存在的结果添加到最终结果集里面
    pub(crate) fn merge(&mut self, other: &LongTimeSelectResult) {
        let mut only_one = Vec::<SingleLongTimeSelectResult>::new();
        for other_item in &other.select_rst {
            let mut contain = false;
            for self_item in &mut self.select_rst {
                if self_item.ts_code == other_item.ts_code {
                    self_item.level = self_item.level + other_item.level;
                    if self_item.level > 100 {
                        self_item.level = 100;
                    }
                    contain = true;
                    break;
                }
            }
            if !contain {
                only_one.push(other_item.clone());
            }
        }
        if !only_one.is_empty() {
            self.select_rst.append(&mut only_one);
        }
        self.ts = Local::now();
    }

    /// 蒋某次选择结果汇总到最终结果中来
    /// @return 返回所有在这一个append当中可买入的股票
    pub(crate) fn append(&mut self, other: &LongTimeSelectResult) -> Vec<String> {
        let mut ret_rst = Vec::<String>::new();
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let short_time_buy_level = config.short_buy_level;
        let mut only_one = Vec::<SingleLongTimeSelectResult>::new();
        for other_item in &other.select_rst {
            let mut contain = false;
            for self_item in &mut self.select_rst {
                if self_item.ts_code == other_item.ts_code {
                    // 小于short_time_buy_level就等于没买过，
                    // 原先没买过，但是新的选择结果其买入等级level更高，那么就将结果值赋成更高的level
                    if self_item.level < short_time_buy_level && self_item.level < other_item.level {
                        self_item.level = other_item.level;
                        self_item.curr_price = other_item.curr_price;
                        self_item.source = other_item.source.clone();
                        ret_rst.push(String::from(&self_item.ts_code));
                    }
                    // 已经买入过了
                    else if self_item.level >= short_time_buy_level {
                        if self_item.level < other_item.level {
                            self_item.level = other_item.level;
                        }
                    }
                    // 对于self_item.level > other_item.level，一概不处理
                    self_item.line_style = other_item.line_style;
                    contain = true;
                    break;
                }
            }
            if !contain {
                let temp_val = other_item.clone();
                if temp_val.level >= short_time_buy_level {
                    ret_rst.push(String::from(&temp_val.ts_code));
                }
                only_one.push(temp_val);
            }
        }
        if !only_one.is_empty() {
            self.select_rst.append(&mut only_one);
        }
        self.ts = Local::now();
        ret_rst
    }

    /// 蒋结果同步到数据库当中去
    pub(crate) async fn sync_to_db(&self) {
        if self.select_rst.is_empty() {
            return;
        }

        let sql_client = crate::initialize::MYSQL_POOL.get().unwrap();
        let mut conn = sql_client.acquire().await.unwrap();
        for item in &self.select_rst {
            let curr_time_str= self.ts.format("%Y%m%d %H:%M:%S").to_string();
            item.insert_to_db(&curr_time_str, &mut conn).await;
        }
    }

    /// 从数据库当中删除所有的结果
    pub(crate) async fn delete() {
        let pool = crate::initialize::MYSQL_POOL.get().unwrap();
        let sql = "delete from short_select_in_time;";
        sqlx::query(sql).execute(pool).await;
    }
}

impl Clone for LongTimeSelectResult {
    fn clone(&self) -> Self {
        let mut vec: Vec<SingleLongTimeSelectResult> = vec![];
        for item in &self.select_rst {
            vec.push(item.clone());
        }
        LongTimeSelectResult {
            select_rst: vec,
            ts: self.ts.clone()
        }
    }
}

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

impl From<&SingleLongTimeSelectResult> for SingleShortTimeHistory {
    fn from(source: &SingleLongTimeSelectResult) -> Self {
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
        let mut query = sqlx::query("insert into short_time_history(ts_code, in_price, in_time, source, level)\
        values(?,?,?,?,?)");
        query = query.bind(self.ts_code.clone());
        query = query.bind(self.in_price);
        query = query.bind(curr_time.clone());
        query = query.bind(self.source.clone());
        query = query.bind(self.level);
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

impl From<&LongTimeSelectResult> for ShortTimeHistory {
    fn from(source: &LongTimeSelectResult) -> Self {
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
//------------------长期筛选结果集--end---------------------------------------------------------------