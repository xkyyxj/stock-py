use crate::sql;


use pyo3::prelude::*;
use async_std::task;
use sqlx::Row;
use crate::cache::AsyncRedisOperation;

use crate::selector::ShortTimeSelect;

#[pyclass]
pub struct ShortTimeStrategy {
    pub(crate) is_started: bool,
}

#[pymethods]
impl ShortTimeStrategy {
    #[new]
    pub(crate) fn new() -> Self {
        ShortTimeStrategy {
            is_started: false
        }
    }

    #[call]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }

        task::spawn(async {
            let mut select = ShortTimeSelect::new().await;
            select.select().await;
        });

        self.is_started = true;
    }

    /// 清空redis缓存
    pub(crate) fn clear(&self) {
        let columns = vec!["ts_code"];
        task::block_on(async {
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