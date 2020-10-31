
/// 数据库最大连接数
pub(crate) static MYSQL_MAX_CONNECTION: usize = 100;

/// 最大盈利的计算期间
pub(crate) static MAX_WIN_CAL_PERIOD: usize = 30;

/// 历史低值：达到多少天以前的最低价格算作是历史低值区间
pub(crate) static MIN_HISTORY_DOWN_DAYS: usize = 200;

/// 历史低值：比历史最低值高多少仍然算作是历史最低值
pub(crate) static MIN_HISTORY_DOWN_UP_PCT: f64 = 0.05;