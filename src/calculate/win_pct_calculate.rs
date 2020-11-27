use async_std::task;
use crate::sql;
use sqlx::Row;
use crate::file::read_txt_file;

/// 此程序用来计算每个策略的获利情况
/// 如果是实时分析程序没有给出盈利结果，说明没有达到实时分析程序设定的盈利目标，那么这个程序用来盈利结果
// TODO -- 设置可以动态向表里面添加列，计算盈利情况
// 可以写定一个列名称形态，例如形如：win_x，其中x为天数

pub struct CalInfo {
    pub table_name: String,     // 表名
    pk_name: String,        // 主键名称
    in_time_name: String,   // 进入时间字段
    in_price_name: String,  // 进入价格字段
    pub cal_days: Vec::<i64>    // 计算哪些天之前的
}

pub struct CalItem {
    ts_code: String,
    in_price: f64,
    in_time: String,
    pk: i64,
}

pub fn parse_table_info(str: String) -> Vec<CalInfo> {
    let mut infos = Vec::<CalInfo>::new();
    let v: Vec<&str> = str.split('\n').collect();
    for item in v {
        let mut temp_info = CalInfo {
            table_name: "".to_string(),
            pk_name: "".to_string(),
            in_time_name: "".to_string(),
            in_price_name: "".to_string(),
            cal_days: vec![]
        };
        let single_table_info = String::from(item);
        let single_table: Vec<&str> = single_table_info.split(':').collect();
        if single_table.len() < 5 {
            // 说明是错误行
            // TODO -- 可以加个日志
            continue;
        }

        temp_info.table_name = String::from(single_table[0]);
        temp_info.pk_name = String::from(single_table[1]);
        temp_info.in_time_name = String::from(single_table[2]);
        temp_info.in_price_name = String::from(single_table[3]);
        let days = String::from(single_table[4]);
        let day_items: Vec<&str> = days.split(',').collect();
        for day_item in day_items {
            let day_item_str = String::from(day_item);
            temp_info.cal_days.push(String::from(day_item_str.trim()).parse().unwrap());
        }
        infos.push(temp_info);
    }
    infos
}

pub async fn win_calculate() {
    // 第一步：获取表格信息
    let info_str = read_txt_file(String::from("./table_config")).await;
    let infos = parse_table_info(info_str);

    // 第二步：单个表格的计算
    for item in infos {
        task::spawn(win_cal_single_table(item));
    }
}

async fn win_cal_single_table(info: CalInfo) {
    // 第零步：确定最长计算日期是多少天之前
    let mut max_day = 0i64;
    for item in &info.cal_days {
        if *item > max_day {
            max_day = *item;
        }
    }
    // 第一步：查询出所有待计算的股票记录
    let mut all_cal_items = Vec::<CalItem>::new();
    let mut qry_sql = String::from("select ts_code, ") + info.in_time_name.as_str() + ",";
    qry_sql = qry_sql + info.pk_name.as_str() + "," + info.in_price_name.as_str() + "";
    qry_sql = qry_sql + " from " + info.table_name.as_str() + " ";
    qry_sql = qry_sql + " where win_" + max_day.to_string().as_str() + " is null";
    sql::async_common_query(qry_sql.as_str(), |rows| {
        for row in rows {
            let temp_item = CalItem{
                ts_code: row.get("ts_code"),
                in_price: row.get::<'_, f64, &str>(info.in_price_name.as_str()),
                in_time: row.get(info.in_time_name.as_str()),
                pk: row.get::<'_, i64, &str>(info.pk_name.as_str()),
            };
            all_cal_items.push(temp_item);
        }
    }).await;

    // 第二步：计算
    for item in &all_cal_items {
        cal_single_stock(&info, item).await;
    }
}

async fn cal_single_stock(info: &CalInfo, item: &CalItem) {
    // 第零步：确定最长计算时长
    let mut max_day = 0i64;
    for item in &info.cal_days {
        if *item > max_day {
            max_day = *item;
        }
    }
    max_day = max_day + 10; // 稳妥一点，多查询十条数据

    // 第一步：查询出基本信息来
    let mut sql = String::from("select close from stock_base_info where ts_code='");
    sql = sql + item.ts_code.as_str() + "' and trade_date > '" + info.in_time_name.as_str() + "'";
    sql = sql + " order by trade_date limit " + max_day.to_string().as_str();
    let mut close_val = Vec::<f64>::new();
    sql::async_common_query(sql.as_str(), |rows| {
        for row in rows {
            close_val.push(row.get::<'_, f64, &str>("close"))
        }
    }).await;

    // 第二步：开始计算逻辑
    let mut any_calculated = false;
    let in_price = item.in_price;
    sql = String::from("update ") + info.table_name.as_str() + " set ";
    let field_prefix = "win_";
    for item in &info.cal_days {
        let real_item = *item;
        if close_val.len() < real_item as usize || real_item < 1 {
            continue;
        }

        let mut n_win = 0f64;
        // 计算五日盈利（买入当天算第一天，第五天盈利如何）
        let target_close = close_val.get(real_item as usize - 1).unwrap();
        n_win = (target_close - in_price) / in_price;

        let field_name = String::from(field_prefix) + item.to_string().as_str();
        sql = sql + field_name.as_str() + "= case when " + field_name.as_str() + " is null then";
        sql = sql + n_win.to_string().as_str() + " else " + field_name.as_str() + " end,";
        any_calculated = true;
    }
    if !any_calculated {
        return;
    }

    sql.pop(); // 弹出最后一个逗号
    sql = sql + " where " + info.pk_name.as_str() + "='" + item.pk.to_string().as_str() + "'";
    println!("update sql is {}", sql);
    sql::async_common_exe(sql.as_str()).await;
}