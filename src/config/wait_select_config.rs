
/// 每天做多有多少只待选股票进入池中
pub static MAX_WAIT_SELECT_EACH_DAY: i64 = 5;

#[derive(Debug)]
pub struct WaitSelectConfig {
    pub max_wait_select_each_day: i64,
}

impl WaitSelectConfig {
    pub fn new() -> Self {
        WaitSelectConfig {
            max_wait_select_each_day: MAX_WAIT_SELECT_EACH_DAY
        }
    }
}