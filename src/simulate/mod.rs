use crate::selector::CommonSelectRst;
use crate::results::{OpeInfo, DBResult};
use crate::utils::time_utils::curr_date_str;
use crate::sql;
use log::{error, info, warn, trace};

mod common_simulate_rst;

pub struct Simulation {

}

impl Simulation {
    pub(crate) fn new() -> Self {
        Simulation {}
    }

    pub(crate) async fn simulate(&self) {

    }
}

pub struct BuyInfoInsert {
    pub already_insert: Vec<Box<OpeInfo>>,
    pub already_buy_codes: Vec<String>,
}

impl BuyInfoInsert {
    
    pub fn new() -> Self {
        BuyInfoInsert {
            already_insert: vec![],
            already_buy_codes: vec![]
        }
    }
    
    pub fn initialize(&mut self) {
        self.refresh();
    }

    pub fn refresh(&mut self) {
        self.already_insert.clear();
        self.already_buy_codes.clear();
        let mut where_part = String::from(" trade_date like '");
        where_part += curr_date_str("%Y%m%d").as_str();
        where_part += "%'";
        self.already_insert = OpeInfo::query_simulate(Some(where_part));
        for item in &self.already_insert {
            self.already_buy_codes.push(item.ts_code.clone());
        }
    }

    /// 将选择结果写入到ope_simulate当中
    pub async fn write_select_rst_2_simulate(&mut self, rst: &CommonSelectRst) {
        if rst.select_rst.is_empty() {
            return;
        }

        let mut conn = crate::initialize::MYSQL_POOL.get().unwrap().acquire().await.unwrap();
        for item in &rst.select_rst {
            if self.already_buy_codes.contains(&item.ts_code) {
                // 警告一下
                warn!("already buy ts_code : {}", &item.ts_code);
                continue;
            }
            let mut buy_info = OpeInfo::new();
            buy_info.ts_code = item.ts_code.clone();
            buy_info.trade_date = curr_date_str("%Y%m%d %H:%M:%S");
            buy_info.ope_close = item.curr_price;
            buy_info.ope_num = 1000; //TODO -- 此处随便指定了一个买入数量，或者我们可以做一个全局的买入信息
            buy_info.simulate = true;
            buy_info.select_type = item.source.clone();
            buy_info.buy_left_num = 1000;// TOOD -- 此处随上面一起修正一下
            buy_info.ope_flag = "Buy".to_string();
            buy_info.win_mny = 0f64;
            buy_info.win_pct = 0f64;
            sql::insert(&mut conn, buy_info).await;
        }
    }
}

