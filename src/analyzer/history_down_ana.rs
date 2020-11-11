use crate::cache::AsyncRedisOperation;
use crate::results::{HistoryDown, DBResult, TimeIndexBatchInfo, StockBaseInfo, WaitSelect};
use std::collections::HashMap;
use crate::sql;
use async_std::task::sleep;
use chrono::{Duration, Local};
use futures::executor;
use crate::analyzer::SleepDuringStop;

pub struct HistoryDownAnalyzer {
    redis_ope: AsyncRedisOperation,
    history_down_vos: Vec<Box<HistoryDown>>,
    last_day_info: HashMap<String, Box<StockBaseInfo>>,
    sleep_check: SleepDuringStop,
    already_in_wait: Vec<String>
}

impl HistoryDownAnalyzer {
    pub(crate) fn new() -> Self {
        let redis_ope = executor::block_on(async {
            AsyncRedisOperation::new().await
        });
        let mut ret_data = HistoryDownAnalyzer {
            redis_ope,
            history_down_vos: vec![],
            last_day_info: Default::default(),
            sleep_check: SleepDuringStop::new(),
            already_in_wait: vec![]
        };
        ret_data.initialize();
        ret_data
    }

    pub(crate) fn initialize(&mut self) {
        self.refresh_data();
    }

    pub(crate) fn refresh_data(&mut self) {
        // 第一步：查询那些股票已经进入到了待选当中了
        let curr_time_str = crate::utils::time_utils::curr_date_str("%Y%m%d");
        let mut in_wait_where = String::from("in_date='");
        in_wait_where = in_wait_where + curr_time_str.as_str() + "'";
        let all_in_wait = WaitSelect::query(Some(in_wait_where));
        for item in all_in_wait {
            self.already_in_wait.push(String::from(item.ts_code));
        }

        // 更新一下昨天的history_down数据
        let history_down_where = String::from("where in_date=(select in_date from history_down order by in_date desc limit 1)");
        let all_vos = HistoryDown::query(Some(history_down_where));
        self.history_down_vos = all_vos;

        // 更新一下基本信息(当前数据库当中最后一天的信息)
        for item in &self.history_down_vos {
            let query_str = String::from("ts_code='") + item.ts_code.as_str() +
                "' and trade_date=(select trade_date from stock_base_info where ts_code='" +
                item.ts_code.as_str() + "' order by trade_date desc limit 1)";
            let yesterday_info = StockBaseInfo::query(Some(query_str));
            for info in yesterday_info {
                self.last_day_info.insert(String::from(&info.ts_code), info);
            }
        }
    }

    pub(crate) async fn analyze(&mut self) {
        let analyze_time_delta = crate::initialize::CONFIG_INFO.get().unwrap().analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        let curr_time_str = crate::utils::time_utils::curr_date_str("%Y%m%d");
        loop {
            let curr_time = Local::now();
            //self.sleep_check.check_sleep(&curr_time).await;
            let mut wait_select_stock = String::new();
            let mut conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
            for item in &self.history_down_vos {
                // 第一步：从Redis缓存当中取出当前的实时数据，判定是否当前价格是否高于昨天的最高价
                // let mut redis_key = String::from(&item.ts_code);
                // redis_key = redis_key + crate::time::INDEX_SUFFIX;
                // let index_info = self.redis_ope.get::<String, String>(redis_key).await;
                // if let None = index_info {
                //     continue;
                // }
                //
                // let str = index_info.unwrap();
                // let real_batch_index: TimeIndexBatchInfo = str.into();
                let last_info = super::get_last_index_info_from_redis(
                    &mut self.redis_ope, &item.ts_code).await;
                if let None = last_info {
                    continue;
                }

                let mut level: i64 = 0;
                let real_last_info = last_info.unwrap();
                println!("last info open is {}, time is {}", real_last_info.t_open, real_last_info.curr_time);
                if real_last_info.curr_price > real_last_info.y_close {
                    level = level + 1;
                }

                let mut temp_ts_code = String::from(&item.ts_code);
                if let Some(last_day_info) = self.last_day_info.get(temp_ts_code.as_str()) {
                    if real_last_info.curr_price > last_day_info.high {
                        level = level + 1;
                    }
                }

                // 如果没有加入到待选当中才会将其加入到待选当中
                if level > 0 && !self.already_in_wait.contains(&temp_ts_code) {
                    let mut rst = WaitSelect::new();
                    rst.ts_code = temp_ts_code;
                    rst.in_reason = String::from("历史低值实时反弹");
                    rst.in_date = String::from(String::from(&curr_time_str));
                    rst.in_price = real_last_info.curr_price;
                    rst.level = level;
                    self.already_in_wait.push(String::from(&rst.ts_code));
                    sql::insert(&mut conn, rst).await;

                    temp_ts_code = String::from(&item.ts_code);
                    wait_select_stock = wait_select_stock + temp_ts_code.as_str() + ", ";
                }
            }

            // 每两秒获取一次
            let two_seconds_duration = Duration::seconds(analyze_time_delta);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = two_seconds_duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
            // 任务栏弹出提示通知消息
            if wait_select_stock.len() > 0 {
                println!("历史低值区间股票有了！！！！");
                taskbar.show_win_toast(String::from("新的待选股票!"), wait_select_stock);
            }
        }
    }

}