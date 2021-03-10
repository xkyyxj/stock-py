/// 查找历史低值的配置项
/// 历史低值：达到多少天以前的最低价格算作是历史低值区间
pub(crate) static MIN_HISTORY_DOWN_DAYS: usize = 200;

/// 历史低值：比历史最低值高多少仍然算作是历史最低值
/// Go程序验证了一个策略，然后我们通过rust实现一下
pub(crate) static MIN_HISTORY_DOWN_UP_PCT: f64 = 0.00001;

/// 历史低值买入信号配置项
/// 发出买入信号的最低涨幅
pub(crate) static MIN_HISTORY_DOWN_BUY_PCT: f64 = 0.03;

/// 发出买入信号的最高涨幅
pub(crate) static MAX_HISTORY_DOWN_BUY_PCT: f64 = 0.07;

/// 发出卖出信号的EMA下跌天数
pub static EMA_DOWN_SOLD_DAYS: i64 = 4;

#[derive(Debug)]
pub struct HistoryDownConfig {
    pub min_history_down_days: usize,
    pub min_history_down_up_pct: f64,

    pub min_history_down_buy_pct: f64,
    pub max_history_down_buy_pct: f64,

    pub ema_down_sold_days: i64,
}

impl HistoryDownConfig {
    pub fn new() -> Self {
        HistoryDownConfig {
            min_history_down_days: MIN_HISTORY_DOWN_DAYS,
            min_history_down_up_pct: MIN_HISTORY_DOWN_UP_PCT,

            min_history_down_buy_pct: MIN_HISTORY_DOWN_BUY_PCT,
            max_history_down_buy_pct: MAX_HISTORY_DOWN_BUY_PCT,

            ema_down_sold_days: EMA_DOWN_SOLD_DAYS,
        }
    }
}