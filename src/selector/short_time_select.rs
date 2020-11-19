use crate::results::TimeIndexBaseInfo;
use crate::selector::ema_select::{ema_select, ema_select_wrapper};
use futures::Future;
use std::pin::Pin;

pub struct ShortTimeSelect {
}

impl ShortTimeSelect {
    pub(crate) fn new() -> Self {
        let mut ret_val = ShortTimeSelect {
        };
        ret_val
    }

    pub(crate) async fn select(&self) {

    }
}