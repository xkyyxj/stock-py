use crate::sql;
use crate::time::fetch_index_info;
use futures::executor;
use pyo3::prelude::*;
use sqlx::Row;
use crate::cache::AsyncRedisOperation;
use once_cell::sync::OnceCell;
use crate::selector::ShortTimeSelect;

static short_time_select: OnceCell<ShortTimeSelect> = OnceCell::new();

#[pyclass]
pub struct ShortTimeStrategy {
    pub(crate) is_started: bool,
}

#[pymethods]
impl ShortTimeStrategy {
    #[new]
    fn new() -> Self {
        let temp = executor::block_on(ShortTimeSelect::new());
        short_time_select.set(temp);
        ShortTimeStrategy {
            is_started: false
        }
    }

    #[call]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }

        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(short_time_select.into_inner().unwrap().select());

        self.is_started = true;
    }

    /// 清空redis缓存
    pub(crate) fn clear(&self) {
        let columns = vec!["ts_code"];
        executor::block_on(async {
            let mut redis_ope = AsyncRedisOperation::new().await;
            let all_rows = sql::query_stock_list(&columns, " where market in ('主板', '中小板')").await.unwrap();
            for item in &all_rows {
                let mut ts_code: String = item.get("ts_code");
                ts_code = ts_code + crate::time::INDEX_SUFFIX;
                redis_ope.delete(ts_code).await;
            }
        });
    }
}