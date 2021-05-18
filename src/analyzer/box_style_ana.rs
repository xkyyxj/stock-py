use crate::cache::AsyncRedisOperation;
use crate::results::{DBResult, TimeIndexBatchInfo, StockBaseInfo, BoxStyle, QueryInfo};
use std::collections::HashMap;

use async_std::task::sleep;
use chrono::{Duration, Local};
use futures::executor;
use crate::utils::time_utils::SleepDuringStop;

/// TODO -- 可以把这个移动到select模块下面去
pub struct BoxStyleAnalyzer {
    redis_ope: AsyncRedisOperation,
    box_style_vos: Vec<Box<BoxStyle>>,
    last_day_info: HashMap<String, Box<StockBaseInfo>>,
    sleep_check: SleepDuringStop
}

impl BoxStyleAnalyzer {
    pub(crate) fn new() -> Self {
        let redis_ope = executor::block_on(async {
            AsyncRedisOperation::new().await
        });
        let mut ret_data = BoxStyleAnalyzer {
            redis_ope,
            box_style_vos: vec![],
            last_day_info: Default::default(),
            sleep_check: SleepDuringStop::new()
        };
        ret_data.initialize();
        ret_data
    }

    pub(crate) fn initialize(&mut self) {
        self.refresh_data();
    }

    pub(crate) fn refresh_data(&mut self) {
        // 更新一下昨天的history_down数据
        let box_style_where = String::from("where in_date=(select in_date from box_style order by in_date desc limit 1)");
        let mut query_info: QueryInfo = Default::default();
        query_info.table_name = Some(String::from("box_style"));
        query_info.where_part = Some(box_style_where);
        let all_vos = BoxStyle::query(&query_info);
        self.box_style_vos = all_vos;

        // 更新一下基本信息(当前数据库当中最后一天的信息)
        for item in &self.box_style_vos {
            let query_str = String::from("ts_code='") + item.ts_code.as_str() +
                "' and trade_date=(select trade_date from stock_base_info where ts_code='" +
                item.ts_code.as_str() + "' order by trade_date desc limit 1)";
            query_info.table_name = Some(String::from("stock_base_info"));
            query_info.where_part = Some(query_str);
            let yesterday_info = StockBaseInfo::query(&query_info);
            for info in yesterday_info {
                self.last_day_info.insert(String::from(&info.ts_code), info);
            }
        }
    }

    pub(crate) async fn analyze(&mut self) {
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        let _curr_time_str = crate::utils::time_utils::curr_date_str("%Y%m%d");
        loop {
            let wait_select_stock = String::new();
            let curr_time = Local::now();
            self.sleep_check.check_sleep(&curr_time).await;
            let _conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
            for item in &self.box_style_vos {
                // 第一步：从Redis缓存当中取出当前的实时数据，判定是否当前价格是否高于昨天的最高价
                let mut redis_key = String::from(&item.ts_code);
                redis_key = redis_key + crate::time::INDEX_SUFFIX;
                let index_info = self.redis_ope.get::<String, String>(redis_key).await;
                if let None = index_info {
                    continue;
                }

                let str = index_info.unwrap();
                let real_batch_index: TimeIndexBatchInfo = str.into();
                let last_info = real_batch_index.get_last_info();
                if let None = last_info {
                    continue;
                }

                let _real_last_info = last_info.unwrap();
            }

            // 每两秒获取一次
            let two_seconds_duration = Duration::seconds(crate::config::INDEX_INFO_FETCH_DELTA);
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