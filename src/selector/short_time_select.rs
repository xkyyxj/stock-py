use crate::results::TimeIndexBaseInfo;
use crate::selector::ema_select::ema_select;
use futures::Future;

pub struct ShortTimeSelect {
    selectors: Vec<Box<fn(&ShortTimeSelect, String) -> dyn Future<Output=bool>>>
}

impl ShortTimeSelect {
    pub(crate) fn new() -> Self {
        let mut ret_val = ShortTimeSelect {
            selectors: vec![]
        };
        ret_val.selectors.push(Box::new(ema_select));
        ret_val
    }

    pub(crate) async fn select(&self) {
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        for item in self.selectors {
            let ts_code = String::from("haha");
            tokio_runtime.spawn(item(self, ts_code));
        }
    }
}