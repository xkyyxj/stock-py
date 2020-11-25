use crate::results::TimeIndexBaseInfo;
use crate::selector::ema_select::{EMASelect};
use futures::{Future, executor};
use std::pin::Pin;
use crate::utils::time_utils::SleepDuringStop;
use chrono::{DateTime, Local, Duration};
use std::sync::mpsc;
use async_std::task::sleep;
use crate::results::DBResult;
use crate::sql;
use sqlx::query::Query;
use sqlx::{MySql, Row};
use sqlx::mysql::MySqlArguments;
use sqlx::pool::PoolConnection;
use crate::simulate::sync_short_history;

//-------------------短期筛选结果集--------------------------------------------------------------------
pub struct SingleShortTimeSelectResult {
    pub ts_code: String,
    pub ts_name: String,
    pub curr_price: f64,
    pub level: i64,              // 评分：0-100分
    pub source: String,          // 来源系统，通过ema选定还是什么其他指标
    pub level_pct: f64,          // 得分的百分比
    pub line_style: i32,         // 分时线形态：-1 一直下降；0 经历过拐点(先下降后上升)；1 先上升后下降；2 一直上涨；3 一直一个价;4 反复波动
}

impl SingleShortTimeSelectResult {
    pub fn new() -> Self {
        SingleShortTimeSelectResult {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            curr_price: 0.0,
            level: 0,
            source: "".to_string(),
            level_pct: 0.0,
            line_style: 3
        }
    }

    pub async fn insert_to_db(&self, curr_time: &String, conn: &mut PoolConnection<MySql>) {
        let mut query = sqlx::query("insert into short_select_in_time(ts_code, in_price, in_time, source, level, line_style)\
        values(?,?,?,?,?,?)");
        query = query.bind(self.ts_code.clone());
        query = query.bind(self.curr_price);
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

impl Clone for SingleShortTimeSelectResult {
    fn clone(&self) -> Self {
        SingleShortTimeSelectResult {
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

pub struct ShortTimeSelectResult {
    pub select_rst: Vec<SingleShortTimeSelectResult>,
    pub ts: DateTime<Local>,
}

impl ShortTimeSelectResult {

    pub fn new() -> Self {
        ShortTimeSelectResult { select_rst: vec![], ts: Local::now() }
    }

    pub fn add_selected(&mut self, info : SingleShortTimeSelectResult) {
        self.select_rst.push(info);
    }

    /// 合并结果用于多个不同的选择策略的合并，蒋选择结果合并到最终结果当中需要用到append方法
    /// 两个结果的合并，重复的结果得分的简单相加，只在一方存在的结果添加到最终结果集里面
    pub fn merge(&mut self, other: &ShortTimeSelectResult) {
        let mut only_one = Vec::<SingleShortTimeSelectResult>::new();
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
    pub fn append(&mut self, other: &ShortTimeSelectResult) -> Vec<String> {
        let mut ret_rst = Vec::<String>::new();
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let short_time_buy_level = config.short_buy_level;
        let mut only_one = Vec::<SingleShortTimeSelectResult>::new();
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
    pub async fn sync_to_db(&self) {
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
    pub async fn delete() {
        let pool = crate::initialize::MYSQL_POOL.get().unwrap();
        let sql = "delete from short_select_in_time;";
        sqlx::query(sql).execute(pool).await;
    }

    /// 从数据库当中查询所有的结果
    pub async fn query_all() -> Self {
        let pool = crate::initialize::MYSQL_POOL.get().unwrap();
        let all_rows = sqlx::query("select * from short_select_in_time").
            fetch_all(pool).await.unwrap();

        let mut ret_rst = ShortTimeSelectResult::new();
        for row in &all_rows {
            let mut temp_item = SingleShortTimeSelectResult::new();
            temp_item.ts_code = row.get("ts_code");
            temp_item.curr_price = row.get::<'_, f64, &str>("in_price");
            temp_item.line_style = row.get::<'_, i32, &str>("line_style");
            temp_item.source = row.get("source");
            temp_item.level = row.get::<'_, i64, &str>("level");
            ret_rst.select_rst.push(temp_item);
        }
        ret_rst
    }
}

impl Clone for ShortTimeSelectResult {
    fn clone(&self) -> Self {
        let mut vec: Vec<SingleShortTimeSelectResult> = vec![];
        for item in &self.select_rst {
            vec.push(item.clone());
        }
        ShortTimeSelectResult {
            select_rst: vec,
            ts: self.ts.clone()
        }
    }
}
//-------------------短期筛选结果集--end---------------------------------------------------------------

pub struct ShortTimeSelect {
    ema_select: EMASelect,
    sleep_check: SleepDuringStop,
    all_result: ShortTimeSelectResult,
}

impl ShortTimeSelect {
    pub async fn new() -> Self {
        let mut ret_val = ShortTimeSelect {
            ema_select: EMASelect::new().await,
            sleep_check: SleepDuringStop::new(),
            all_result: ShortTimeSelectResult::new()
        };
        ret_val.initialize().await;
        ret_val
    }

    pub async fn initialize(&mut self) {
        self.ema_select.initialize().await;
    }

    pub async fn select(&mut self) {
        // 第零步：获取初始化的配置信息
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let ana_delta_time = config.analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        loop {
            let (tx, rx) = mpsc::channel::<ShortTimeSelectResult>();
            let curr_time = Local::now();
            sync_short_history(curr_time).await;
            // FIXME --　别忘了取消注释
            //self.sleep_check.check_sleep(&curr_time).await;
            let future = self.ema_select.select(tx);
            futures::join!(future);

            let mut temp_result = ShortTimeSelectResult::new();
            for received  in rx {
                // 获取结果
                println!("recerved length is {}", received.select_rst.len());
                temp_result.merge(&received);
            }
            let new_buy_stock = self.all_result.append(&temp_result);
            let mut wait_select_stock = String::from("");
            for item in new_buy_stock {
                wait_select_stock = wait_select_stock + item.as_str();
            }

            // 先从库里面删除，然后再将最新结果添加到数据库当中
            ShortTimeSelectResult::delete().await;
            self.all_result.sync_to_db().await;

            // 任务栏弹出提示通知消息(评分大于等于60就买入吧)
            if wait_select_stock.len() > 0 {
                println!("EMA信号");
                taskbar.show_win_toast(String::from("EMA Select:"), wait_select_stock);
            }

            // 每两秒获取一次
            let two_seconds_duration = Duration::seconds(ana_delta_time);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = two_seconds_duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
        }
    }

    /// 处理策略：如果是
    fn process_ana_result(&mut self, result: ShortTimeSelectResult) {

    }
}