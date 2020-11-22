use crate::results::TimeIndexBaseInfo;
use crate::selector::short_time_select::ShortTimeSelect;
use futures::Future;
use std::pin::Pin;
use crate::sql;
use std::collections::HashMap;
use sqlx::Row;
use crate::cache::{get_num_last_index_info_redis, AsyncRedisOperation};
use crate::selector::{SelectResult, SingleSelectResult};
use std::sync::mpsc::Sender;

static EAM_LEVEL_PCT: f64 = 0.5;

pub struct EMAAnaInfo {
    last_ema_value: f64,

}

pub struct EMASelect {
    backup_codes: Vec<String>,
    selected_rst: SelectResult,
    code2name_map: HashMap<String, String>,
    code2ana_info_map: HashMap<String, EMAAnaInfo>,
    redis_ope: AsyncRedisOperation,
}

impl EMASelect {
    pub(crate) async fn new() -> Self {
        EMASelect {
            backup_codes: vec![],
            selected_rst: SelectResult::new(),
            code2name_map: Default::default(),
            code2ana_info_map: Default::default(),
            redis_ope: AsyncRedisOperation::new().await
        }
    }

    pub(crate) async fn initialize(&mut self) {
        // 第负一步：清空以及获取初始化信息
        self.backup_codes.clear();
        self.code2ana_info_map.clear();
        self.code2ana_info_map.clear();
        let config_info = crate::initialize::CONFIG_INFO.get().unwrap();
        // 第零步：查询出所有的股票列表
        let columns = vec!["ts_code", "name"];
        let query_list_fut = sql::query_stock_list(&columns, "");
        let stock_list = query_list_fut.await.unwrap();

        // 第一步：查询出所有的EMA开始上扬的股票，然后放到备胎当中
        let ema_length = config_info.ema_select_length;
        let ema_up_days = config_info.ema_select_up_days;
        let mut ema_field = String::from("ema_");
        ema_field = ema_field + ema_length.to_string().as_str();
        for item in stock_list {
            let ts_code: String = item.get("ts_code");
            let ts_name: String = item.get("name");
            // 第一点一步：查询ema_value，确定是否连续N天上涨
            let mut query_str = String::from("select trade_date, ");
            query_str = query_str + ema_field.as_str() + " from ema_value ";
            query_str = query_str + " where ts_code='" + ts_code.as_str() + "'";
            query_str = query_str + " order by trade_date desc limit ";
            query_str = query_str + ema_up_days.to_string().as_str();
            let mut is_always_up = true;
            let mut pre_date = String::from("0000-00-00");
            let mut pre_ema_val = 0f64;
            sql::async_common_query(query_str.as_str(), |rows| {
                for item in rows {
                    let trade_date: String = item.get("trade_date");
                    let ema_val = item.get::<'_, f64, &str>(ema_field.as_str());
                    // 严格上涨
                    if is_always_up && trade_date > pre_date && ema_val > pre_ema_val {
                        pre_date = trade_date;
                        pre_ema_val = ema_val;
                    }
                    else if trade_date > pre_date {
                        is_always_up = false;
                    }
                }
            }).await;

            // 第一点二步：如果是持续上涨的话，那么就加入到备胎当中去
            self.backup_codes.push(String::from(&ts_code));
            self.code2name_map.insert(String::from(&ts_code), ts_name);
            let ema_ana_info = EMAAnaInfo{ last_ema_value: pre_ema_val };
            self.code2ana_info_map.insert(ts_code, ema_ana_info);
        }
    }

    /// 策略：获取最近的几条实时信息，如果是正处于下降过程当中的，那么就不加入到备选当中，如果是经历过拐点的，加入到备选当中
    /// 如果是一直处于上涨的过程当中，给个中等评分吧
    pub(crate) async fn select(&mut self, tx: Sender<SelectResult>) {
        for i in 0..self.backup_codes.len() {
            let item = self.backup_codes.get(i).unwrap();
            let temp_ts_code = String::from(item);
            let redis_info = get_num_last_index_info_redis(
                &mut self.redis_ope, &temp_ts_code, 5).await;
            if let None = redis_info {
                return;
            }

            let real_redis_info = redis_info.unwrap();
            // -1 一直下降；0 经历过拐点(先下降后上升)；1 先上升后下降；2 一直上涨；3 一直一个价;4 反复波动
            let mut line_type = 3;
            let mut pre_price = real_redis_info.get(0).unwrap().curr_price;
            for item in &real_redis_info {
                let curr_price = item.curr_price;
                line_type = EMASelect::judge_next_state(line_type, pre_price, curr_price);
            }

            // TODO -- 待完善
            let single_rst = SingleSelectResult{
                ts_code: String::from(&temp_ts_code),
                ts_name: String::from(self.code2name_map.get(temp_ts_code.as_str()).unwrap()),
                curr_price: pre_price,
                level: 0,
                source: String::from("EMA Select"),
                level_pct: 0.0,
                line_style: line_type
            };
            self.judge_can_add(single_rst, line_type);
            tx.send(self.selected_rst.clone());
        }
    }

    fn judge_can_add(&mut self, mut single_rst: SingleSelectResult, up_state: i32) {
        match up_state {
            -1 => {
                single_rst.level = 0;
            },
            0 => {
                single_rst.level = 90;
                self.selected_rst.add_selected(single_rst);
            },
            1 => {
                single_rst.level = 10;
                self.selected_rst.add_selected(single_rst);
            },
            2 => {
                single_rst.level = 60;
                self.selected_rst.add_selected(single_rst);
            },
            4 => {
                single_rst.level = 40;
                self.selected_rst.add_selected(single_rst);
            },
            _ => {}
        }
    }

    fn judge_next_state(mut pre_state: i32, pre_price: f64, curr_price: f64) -> i32 {
        match pre_state {
            -1 => {
                if curr_price > pre_price {
                    pre_state = 0;
                }
            },
            0 => {
                if curr_price < pre_price {
                    pre_state = 4;
                }
            },
            1 => {
                if curr_price > pre_price {
                    pre_state = 4;
                }
            },
            2 => {
                if curr_price < pre_price {
                    pre_state = 1;
                }
            },
            3 => {
                if curr_price < pre_price {
                    pre_state = -1;
                }
                else if curr_price > pre_price {
                    pre_state = 2;
                }
            },
            _ => {}
        }
        pre_state
    }
}