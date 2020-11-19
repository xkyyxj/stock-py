use crate::results::TimeIndexBaseInfo;
use crate::selector::short_time_select::ShortTimeSelect;
use futures::Future;
use std::pin::Pin;

pub fn ema_select_wrapper(selector: &ShortTimeSelect, ts_code: String) -> Pin<Box<dyn Future<Output=bool> + Send + 'static>> {
    Box::pin(ema_select(selector, ts_code))
}

pub async fn ema_select(selector: &ShortTimeSelect, ts_code: String) -> bool {
    true
}