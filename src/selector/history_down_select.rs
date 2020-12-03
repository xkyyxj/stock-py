use std::collections::HashMap;

use crate::sql;
use sqlx::Row;
use crate::cache::{get_num_last_index_info_redis, AsyncRedisOperation};
use crate::results::TimeIndexBaseInfo;
use crate::selector::{CommonSelectRst, SingleCommonRst, SHORT_TYPE, FINAL_TYPE, LONG_TYPE};
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use chrono::Local;

pub struct HistoryDownAnaInfo {
    ts_code: String,
    history_down_price: f64,
    pk_history_down: i64,
}

pub struct HistoryDownSelect {
    selected: Vec::<String>,
    backup: Vec::<HistoryDownAnaInfo>,
    redis_ope: AsyncRedisOperation,
    initialized: bool,
}

impl HistoryDownSelect {
    pub async fn new() -> Self {
        HistoryDownSelect {
            selected: vec![],
            backup: vec![],
            redis_ope: AsyncRedisOperation::new().await,
            initialized: false
        }
    }

    pub async fn initialize(&mut self) {
        // 第零步：查询最新的history_down表中的记录
        let mut sql = String::from("select in_date from history_down order by in_date desc limit 1");
        let mut last_date_str: String = String::new();
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                last_date_str = row.get("in_date");
            }
        }).await;

        sql = String::from("select * from history_down where in_date='");
        sql = sql + last_date_str.as_str() + "'";
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                let temp_info = HistoryDownAnaInfo {
                    ts_code: row.get("ts_code"),
                    history_down_price: row.get::<'_, f64, &str>("his_down_price"),
                    pk_history_down: row.get::<'_, i64, &str>("pk_history_down"),
                };
                self.backup.push(temp_info);
            }
        }).await;
        self.initialized = true;
    }

    pub fn get_name() -> String {
        return String::from("history_down_select");
    }

    /// 判定逻辑有如下几点（实时分析程序就好了）：
    /// 1. 最新价格比历史最低价上涨幅度在config信息里面标注的幅度之间
    /// 2. 当前价格正在上涨过程当中
    /// 3. 比昨天的收盘价要高
    pub(crate) async fn select(&mut self, mut tx: UnboundedSender<CommonSelectRst>) -> () {
        if !self.initialized {
            return;
        }

        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let min_up_pct = config.history_down_config.min_history_down_buy_pct;
        let max_up_pct = config.history_down_config.max_history_down_buy_pct;

        let mut selected_rst = CommonSelectRst::new();
        for item in &self.backup {
            // 第一步：获取最新的redis缓存信息
            let temp_ts_code = String::from(&item.ts_code);
            let redis_info = get_num_last_index_info_redis(
                &mut self.redis_ope, &temp_ts_code, 5).await;
            if let None = redis_info {
                continue;
            }

            // 第二步：计算并且判定
            let mut selected = true;
            let real_redis_info = redis_info.unwrap();
            let last_info = real_redis_info.last().unwrap();
            let last_price = last_info.curr_price;
            // 1. 最新价格比历史最低价上涨幅度在config信息里面标注的幅度之间
            let up_pct = (last_price - item.history_down_price) / item.history_down_price;
            selected = selected && up_pct > min_up_pct && up_pct < max_up_pct;
            if !selected {
                continue;
            }
            // 2. 当前价格正在上涨过程当中
            selected = selected && judge_is_up(&real_redis_info);
            if !selected {
                continue;
            }
            // 3. 比昨天的收盘价要高
            let pre_day_close = last_info.y_close;
            selected = selected && last_price > pre_day_close;
            if !selected {
                continue;
            }

            // 第三步：如果成功了，更新history_down的selected字段，并且添加到选中结果集当中去
            let mut sql = String::from("update history_down set selected='Y' where pk_history_down='");
            sql = sql + item.pk_history_down.to_string().as_str() + "'";
            sql::async_common_exe(sql.as_str()).await;
            // TODO -- level以及ts_name字段都没有赋值
            let single_rst = SingleCommonRst {
                ts_code: String::from(&item.ts_code),
                ts_name: "".to_string(),
                curr_price: last_price,
                level: 0,
                source: "history_down".to_string(),
                level_pct: 0.0,
                line_style: 0,
                rst_style: SHORT_TYPE & LONG_TYPE & FINAL_TYPE
            };
            selected_rst.add_selected(single_rst);
        }
        tx.send(selected_rst).await;
    }
}

fn judge_is_up(redis_info: &Vec::<TimeIndexBaseInfo>) -> bool {
    let pre_price = redis_info.get(0).unwrap().curr_price;
    for item in redis_info {
        let curr_price = item.curr_price;
        if curr_price > pre_price {
            return false;
        }
    }
    true
}