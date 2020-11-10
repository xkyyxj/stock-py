
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

#[derive(Debug)]
pub struct Config {
    pub(crate) mysql_max_connection: usize,
    pub(crate) max_win_cal_period: usize,
    pub(crate) min_history_down_days: usize,
    pub(crate) min_history_down_up_pct: f64,
    pub(crate) index_info_fetch_delta: i64,
    pub(crate) analyze_time_delta: i64,
}

impl Config {
    pub(crate) fn new() -> Self {
        Config {
            mysql_max_connection: MYSQL_MAX_CONNECTION,
            max_win_cal_period: MAX_WIN_CAL_PERIOD,
            min_history_down_days: MIN_HISTORY_DOWN_DAYS,
            min_history_down_up_pct: MIN_HISTORY_DOWN_UP_PCT,
            index_info_fetch_delta: INDEX_INFO_FETCH_DELTA,
            analyze_time_delta: ANALYZE_TIME_DELTA
        }
    }

    pub(crate) fn set_mysql_max_connection(&mut self, conn_num: usize) {
        self.mysql_max_connection = conn_num;
    }

    pub(crate) fn set_index_info_fetch_delta(&mut self, delta: i64) {
        self.index_info_fetch_delta = delta;
    }
}