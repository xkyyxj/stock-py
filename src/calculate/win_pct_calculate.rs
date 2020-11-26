use crate::sql;
use sqlx::Row;

/// 此程序用来计算每个策略的获利情况
/// 如果是实时分析程序没有给出盈利结果，说明没有达到实时分析程序设定的盈利目标，那么这个程序用来盈利结果
//TODO -- 或许可以写一个配置文件，计算的表，多少天盈利，都写进去
// TODO -- 设置可以动态向表里面添加列，计算盈利情况
// 可以写定一个列名称形态，例如形如：win_x，其中x为天数

struct CalInfo {
    table_name: String,     // 表名
    pk_name: String,        // 主键名称
    in_time_name: String,   // 进入时间字段
    in_price_name: String,  // 进入价格字段
    cal_days: Vec::<i64>    // 计算哪些天之前的
}

pub async fn win_calculate() {

}

async fn win_cal_single_table(info: CalInfo) {
    // 第一步：查询出所有待计算的股票记录
    let mut qry_sql = String::from("select ts_code, ");
    qry_sql = qry_sql + info.pk_name.as_str() + "," + info.in_price_name.as_str() + "";
    qry_sql = qry_sql + " from " + info.table_name.as_str() + " ";
    // qry_sql = qry_sql + " where " + info.in_time_name.as_str() " "
}

async fn cal_single_stock(info: &CalInfo, ts_code: String) {

    let mut sql = String::from("select close from stock_base_info where ts_code='");
    sql = sql + ts_code.as_str() + "' and trade_date > '" + in_time.as_str() + "'";
    sql = sql + " order by trade_date limit 20";
    let mut close_val = Vec::<f64>::new();
    sql::async_common_query(sql.as_str(), |rows| {
        for row in rows {
            close_val.push(row.get::<'_, f64, &str>("close"))
        }
    }).await;

    let mut five_win = 0f64;
    let mut seven_win = 0f64;
    let mut update_five = false;
    let mut update_seven = false;
    if close_val.len() > 4 {
        // 计算五日盈利（买入当天算第一天，第五天盈利如何）
        let target_close = close_val.get(3).unwrap();
        five_win = (target_close - in_price) / in_price;
        update_five = true;
    }

    if close_val.len() > 6 {
        // 计算五日盈利（买入当天算第一天，第五天盈利如何）
        let target_close = close_val.get(5).unwrap();
        seven_win = (target_close - in_price) / in_price;
        update_seven = true;
    }

    if !update_five && !update_seven {
        return;
    }

    sql = String::from("update short_time_history set win_5='");
    sql = sql + five_win.to_string().as_str() + "'";
    if update_seven {
        sql = sql + ", win_7='" + seven_win.to_string().as_str() + "'"
    }
    sql = sql + " where pk_short_history='" + pk.to_string().as_str() + "'";
    sql::async_common_exe(sql.as_str()).await;
}