use crate::sql;
use sqlx::Row;
use crate::cache::{get_num_last_index_info_redis, AsyncRedisOperation};
use crate::results::{TimeIndexBaseInfo, HistoryDown};
use crate::selector::{CommonSelectRst, SingleCommonRst, SHORT_TYPE, LONG_TYPE};
use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;

pub struct VolSelect {
    vol_up_codes: Vec::<String>, // 4日交易量移动平均开始上涨的股票
    boom_codes: Vec::<String>,  // 爆发式提升成交量的股票编码
    initialized: bool,
}

impl VolSelect {

    pub async fn new() -> Self {
        VolSelect {
            vol_up_codes: vec![],
            boom_codes: vec![],
            initialized: false
        }
    }

    pub async fn initialize(&mut self) {
        // 第零步：查询最新的vol_ema表中的记录
        let mut sql = String::from("select trade_date from vol_ema order by trade_date desc limit 1");
        let mut last_date_str: String = String::new();
        sql::async_common_query(sql.as_str(), |rows| {
            for row in rows {
                last_date_str = row.get("trade_date");
            }
        }).await;

        // 第一步：过滤所有的4日交易量的移动平均开始上涨的股票
        sql = String::from("select * from vol_ema where trade_date='");
        sql = sql + last_date_str.as_str() + "'";

    }

}