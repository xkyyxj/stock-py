
/// 数据库最大连接数
pub(crate) static MYSQL_MAX_CONNECTION: usize = 100;

/// 最大盈利的计算期间
pub(crate) static MAX_WIN_CAL_PERIOD: usize = 30;

/// 历史低值：达到多少天以前的最低价格算作是历史低值区间
pub(crate) static MIN_HISTORY_DOWN_DAYS: usize = 200;

/// 历史低值：比历史最低值高多少仍然算作是历史最低值
pub(crate) static MIN_HISTORY_DOWN_UP_PCT: f64 = 0.05;

/// 实时信息的获取间隔（秒钟）
pub(crate) static INDEX_INFO_FETCH_DELTA: i64 = 3;

/// 分析程序分析时间间隔(秒钟)
pub(crate) static ANALYZE_TIME_DELTA: i64 = 4;

/// 空中楼阁理论：上涨了多少算作是已经开启上涨了
pub(crate) static AIR_CASTLE_UP_PCT: f64 = 0.15;

/// 空中楼阁理论：连续多少天上涨了才算开启上涨了
pub(crate) static AIR_CASTLE_UP_DAYS: i64 = 5;

/// 短期选股用到的参数---------------------------------------------------------------------------------
/// 当短期选股的评分达到了多少值得时候就买入（0-100）
pub(crate) static SHORT_BUY_LEVEL: i64 = 60;

/// ema短期选股需要用到的参数---------------------------------------------------------------------------
/// 无意之中通过python的ema模拟发现，其实ema指标可以实现盈利，当时是用的5，此处也用5吧
pub(crate) static EMA_SELECT_DEFAULT_LENGTH: i64 = 5;

/// 当EMA连续多少天上涨之后就决定买入！！
pub(crate) static EMA_SELECT_UP_DAYS: i64 = 3;

#[derive(Debug)]
pub struct Config {
    pub(crate) mysql_max_connection: usize,
    pub(crate) max_win_cal_period: usize,
    pub(crate) min_history_down_days: usize,
    pub(crate) min_history_down_up_pct: f64,
    pub(crate) index_info_fetch_delta: i64,
    pub(crate) analyze_time_delta: i64,
    pub(crate) air_castle_up_pct: f64,
    pub(crate) air_castle_up_days: i64,
    pub(crate) ema_select_length: i64,
    pub(crate) ema_select_up_days: i64,
    pub(crate) short_buy_level: i64,
}

impl Config {
    pub(crate) fn new() -> Self {
        Config {
            mysql_max_connection: MYSQL_MAX_CONNECTION,
            max_win_cal_period: MAX_WIN_CAL_PERIOD,
            min_history_down_days: MIN_HISTORY_DOWN_DAYS,
            min_history_down_up_pct: MIN_HISTORY_DOWN_UP_PCT,
            index_info_fetch_delta: INDEX_INFO_FETCH_DELTA,
            analyze_time_delta: ANALYZE_TIME_DELTA,
            air_castle_up_pct: AIR_CASTLE_UP_PCT,
            air_castle_up_days: AIR_CASTLE_UP_DAYS,
            ema_select_length: EMA_SELECT_DEFAULT_LENGTH,
            ema_select_up_days: EMA_SELECT_UP_DAYS,
            short_buy_level: SHORT_BUY_LEVEL
        }
    }

    pub(crate) fn set_mysql_max_connection(&mut self, conn_num: usize) {
        self.mysql_max_connection = conn_num;
    }

    pub(crate) fn set_index_info_fetch_delta(&mut self, delta: i64) {
        self.index_info_fetch_delta = delta;
    }
}