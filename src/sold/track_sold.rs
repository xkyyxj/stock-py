use crate::sql;
use crate::results::{OpeInfo, WaitSold};
use crate::utils::time_utils::curr_date_str;
use crate::cache::{ AsyncRedisOperation, get_last_index_info_from_redis };
use sqlx::{Row, MySql};
use crate::sold::history_down_policy::history_down_judge;
use crate::utils::time_utils::SleepDuringStop;
use log::{error, info, warn};
use sqlx::pool::PoolConnection;
use chrono::Local;
use async_std::task::{self, sleep};
use chrono::Duration;
use crate::sold::SoldInfo;

pub static HISTORY_DOWN_TYPE: &str = "down";

pub struct TrackSold {
    pub real_hold: Vec<OpeInfo>,
    pub simulate_hold: Vec<OpeInfo>,
    pub time_check: SleepDuringStop,
}

impl TrackSold {

    pub fn new() -> Self {
        TrackSold {
            real_hold: vec![],
            simulate_hold: vec![],
            time_check: SleepDuringStop::new(),
        }
    }

    pub async fn initialize(&mut self) {
        self.refresh_data().await;
    }

    pub async fn refresh_data(&mut self) {
        self.real_hold.clear();
        self.simulate_hold.clear();
        // 从数据库当中查询出来所有的买入信息
        let mut count = 0;
        let tables:[&str;2] = ["operate_info", "operate_info_simulate"];
        for table in &tables {
            let mut sql = String::from("select * from ") + table + " where buy_left_num>0";
            sql::async_common_query(sql.as_str(), |set| {
                for item in set {
                    let has_sold: String = item.get("has_sold");
                    let temp_info = OpeInfo {
                        pk_ope: item.get::<'_, i64, &str>("pk_ope"),
                        ts_code: item.get("ts_code"),
                        trade_date: item.get("trade_date"),
                        ope_num: item.get::<'_, i64, &str>("ope_num"),
                        ope_close: item.get::<'_, f64, &str>("ope_close"),
                        ope_flag: item.get("ope_flag"),
                        win_mny: item.get::<'_, f64, &str>("win_mny"),
                        win_pct: item.get::<'_, f64, &str>("win_pct"),
                        select_type: item.get("select_type"),
                        pk_buy_ope: item.get::<'_, i64, &str>("pk_buy_ope"),
                        buy_left_num: item.get::<'_, i64, &str>("buy_left_num"),
                        simulate: false
                    };
                    if count == 0 {
                        self.real_hold.push(temp_info);
                    } else {
                        self.simulate_hold.push(temp_info);
                    }
                }
            }).await;
        }
    }

    pub async fn sold(&mut self) {
        // 第零步：获取初始化的配置信息
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let ana_delta_time = config.analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        loop {
            warn!("Track Sold loop!");
            let curr_time = Local::now();
            self.time_check.check_sleep(&curr_time).await;
            let mut conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
            for item in &mut self.real_hold {
                self.sold_item(item, false, &mut conn).await;
            }

            for item in &mut self.simulate_hold {
                self.sold_item(item, true, &mut conn).await;
            }

            // 每X秒获取一次(由analyze_time_delta指定)
            let duration = Duration::seconds(ana_delta_time);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
        }
    }

    // TODO -- 缺少对于卖出比例的处理，现在权当是全仓卖出
    pub async fn sold_item(&self, buy_info: &OpeInfo, is_simulate: bool, conn: &mut PoolConnection<MySql>) {
        let mut select_type = "".to_string();
        let judge_rst = match buy_info.select_type.as_str() {
            "down" => {
                select_type = "Down".to_string();
                history_down_judge(buy_info).await
            },
            _ => {}
        };

        if !judge_rst.can_sold {
            return;
        }
        let mut redis_ope = AsyncRedisOperation::new().await;
        let last_info = get_last_index_info_from_redis(&mut redis_ope, &buy_info.ts_code).await;
        if let None = last_info {
            error!("Track Sold : Can not get redis cache —— {}", buy_info.ts_code);
            return;
        }

        let real_info = last_info.unwrap();
        // 计算相关的比例
        let win_pct = (real_info.curr_price - buy_info.ope_close) / buy_info.ope_close;
        let win_mny = (real_info.curr_price - buy_info.ope_close) * buy_info.ope_num;
        // 开始做卖出操作（FIXME -- 此处似乎完全没有考虑事务的问题？？？？）
        if is_simulate {
            let ope_info = OpeInfo {
                pk_ope: 0,
                ts_code: String::from(&buy_info.ts_code),
                trade_date: buy_info.trade_date.clone(),
                ope_num: buy_info.ope_num,
                ope_close: real_info.curr_price,
                ope_flag: "Sold".to_string(),
                win_mny,
                win_pct,
                select_type,
                pk_buy_ope: buy_info.pk_ope,
                buy_left_num: 0,
                simulate: true
            };
            sql::insert(conn, ope_info).await;

            // 更正买入记录（TODO -- 此处当作是全部卖出了，部分卖出以后再说）
            let mut sql_str = "update operate_info_simulate set buy_left_num=0 where pk_ope='";
            sql_str += buy_info.pk_ope + "'";
            sql::async_common_exe(sql_str).await;
        } else {
            // 给出提示信息，插入到wait_sold这张表格当中
            let mut wait_sold = WaitSold::new();
            wait_sold.ts_code = buy_info.ts_code.clone();
            wait_sold.trade_date = curr_date_str("%Y%m%d %H:%M:%S");
            wait_sold.ope_num = buy_info.ope_num;
            wait_sold.ope_close = real_info.curr_price;
            wait_sold.win_mny = win_mny;
            wait_sold.win_pct = win_pct;
            wait_sold.select_type = select_type;
            wait_sold.pk_buy_ope = buy_info.pk_ope;
            sql::insert(conn, wait_sold).await;
        }
    }
}