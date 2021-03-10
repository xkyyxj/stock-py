use crate::sql;
use crate::results::{ HoldInfo };
use sqlx::Row;

pub struct TrackSold {
    pub real_hold: Vec<HoldInfo>,
    pub simulate_hold: Vec<HoldInfo>,
}

impl TrackSold {

    pub fn new() -> Self {
        TrackSold {
            real_hold: vec![],
            simulate_hold: vec![]
        }
    }

    pub async fn initialize(&mut self) {
        // 从数据库当中查询出来所有的买入信息
        let mut count = 0;
        let tables:[&str;2] = ["operate_info", "operate_info_simulate"];
        for table in &tables {
            let mut sql = String::from("select * from ") + table + " where has_sold='N'";
            sql::async_common_query(sql.as_str(), |set| {
                for item in set {
                    let has_sold: String = item.get("has_sold");
                    let temp_info = HoldInfo {
                        pk_ope: item.get::<'_, i64, &str>("pk_ope"),
                        ts_code: item.get("ts_code"),
                        trade_date: item.get("trade_date"),
                        ope_num: item.get::<'_, i64, &str>("ope_num"),
                        ope_close: item.get::<'_, f64, &str>("ope_close"),
                        has_sold: has_sold == "Y",
                        ope_flag: item.get("ope_flag"),
                        sold_close: item.get::<'_, f64, &str>("sold_close"),
                        sold_date: item.get("sold_date"),
                        win_mny: item.get::<'_, f64, &str>("win_mny"),
                        win_pct: item.get::<'_, f64, &str>("win_pct"),
                        select_type: item.get("select_type"),
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

    }
}