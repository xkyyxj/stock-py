use crate::results::TimeIndexBaseInfo;
use crate::selector::short_time_select::ShortTimeSelect;
use futures::Future;
use std::pin::Pin;
use crate::sql;
use std::collections::HashMap;
use sqlx::Row;

pub struct AnaInfo {

}

pub struct EMASelect {
    backup_codes: Vec<String>,
    selectd_codes: Vec<String>,
    code2name_map: HashMap<String, String>,
    code2ana_info_map: HashMap<String, AnaInfo>,
    ema_length: i64,
}

impl EMASelect {
    pub(crate) fn new() -> Self {
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        EMASelect {
            backup_codes: vec![],
            selectd_codes: vec![],
            code2name_map: Default::default(),
            code2ana_info_map: Default::default(),
            ema_length: config.ema_select_length
        }
    }

    pub(crate) async fn initialize(&mut self) {
        // 第零步：查询出所有的股票列表
        let columns = vec!["ts_code", "name"];
        let query_list_fut = sql::query_stock_list(&columns, "");
        let stock_list = query_list_fut.await.unwrap();

        // 第一步：查询出所有的EMA开始上扬的股票，然后放到备胎当中
        let ema_feild = String::from("ema_");
        ema_field = ema_feild + self.ema_length.to_string().as_str();
        for item in stock_list {
            let ts_code: String = item.get("ts_code");
            let ts_name: String = item.get("name");
            // 第一点一步：查询ema_value，确定是否连续N天上涨
            let mut query_str = String::from("select trade_date, ");
            query_str = query_str + ema_field.as_str() + " from ema_value ";
            query_str = query_str + " where ts_code='" + ts_code.as_str() + "'";
            query_str = query_str + " order by trade_date desc limit ";
            query_sql = query_str + self.ema_length.to_string().as_str();
            sql::async_common_query(query_str.as_str(), |rows| {
                for item in rows {
                    let trade_date = item.get("trade_date");
                    let ema_val = item.get(ema_feild.as_str());
                }
            }).await;
        }
    }

    pub(crate) async fn select(&mut self) {

    }
}