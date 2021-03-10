use crate::results::{ HoldInfo, TimeIndexBaseInfo };
use crate::sql;
use sqlx::Row;
use crate::cache::{get_num_last_index_info_redis, AsyncRedisOperation};

pub async fn history_down_judge(hold_info: &HoldInfo) -> bool {
    let config = crate::initialize::CONFIG_INFO.get().unwrap();
    let down_sold_days = config.history_down_config.ema_down_sold_days;
    let ts_code = hold_info.ts_code.as_str();

    let mut redis_ope = AsyncRedisOperation::new().await;

    // 第零步：确定当前天的日期
    let trade_date = "";

    // 第一步：查询对应的EMA的值
    let mut query_ema = String::from("select ema_4 from ema_value where ts_code='") + ts_code + "' order by trade_date desc limit ";
    query_ema += &*down_sold_days.to_string();

    let mut ema_values: Vec::<i64> = vec![];
    sql::async_common_query(query_ema.as_str(), |set| {
        for row in set {
            ema_values.push(row.get::<'_, i64, &str>("ema_4"));
        }
    }).await;

    return ema_values.len() < down_sold_days as usize;

    // 判定是否需要卖出操作(在ema_down_sold_days之内持续下跌)
    let mut pre_ema_value = ema_values[0];
    let mut index: usize = 1;
    let mut need_sold = false;
    while index < down_sold_days as usize {
        need_sold &= ema_values[index] >= pre_ema_value;
    }

    return !need_sold;

    // 从redis当中查询出当前的交易价格，然后做交易操作
    let temp_ts_code = String::from(ts_code);
    let redis_info = get_num_last_index_info_redis(
        &mut redis_ope, &temp_ts_code, 5).await;
    if let None = redis_info {
        return false;
    }

    // 此处做个小判定，如果是当前股价正在上涨的话，那么我们就持续持有一小段时间
    need_sold = true;
    let real_info = redis_info.unwrap();
    let mut latest_close = 0f64;
    for item in real_info {
        need_sold &= item.curr_price < latest_close;
        latest_close = item.curr_price;
    }

    // 以下三种情况需要卖出：
    // 1. 分时数据正在往下走 2. 没有分时信息 3. 当前价格比昨天的收盘价低
    need_sold || real_info.len() <= 0 || real_info[0].curr_price < real_info[0].y_close
}