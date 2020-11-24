mod ema_select;
mod short_time_select;
mod sold_policy;
mod long_time_select;

pub use short_time_select::{ShortTimeSelect, ShortTimeSelectResult, SingleShortTimeSelectResult};

static ALWAYS_DOWN: i32 = -1;       // 一直下降
static DOWN_THEN_UP: i32 = 0;       // 经历过拐点(先下降后上升)
static UP_THEN_DOWN: i32 = 2;       // 先上升后下降
static ALWAYS_UP: i32 = 2;          // 一直上涨
static SINGLE_PRICE: i32 = 3;       // 一直一个价格
static WAVE: i32 = 4;               // 反复波动


