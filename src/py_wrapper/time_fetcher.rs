use crate::sql;
use crate::time::fetch_index_info;
use futures::executor;
use pyo3::prelude::*;
use sqlx::Row;

// 每个线程负责拉取的股票数量
static EACH_THREAD_FETCH_NUM: usize = 330;

#[pyclass]
pub struct TimeFetcher {
    pub(crate) is_started: bool
}

#[pymethods]
impl TimeFetcher {
    #[new]
    fn new() -> Self {
        TimeFetcher { is_started: false }
    }

    #[call]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }

        let columns = vec!["ts_code"];
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        let stock_codes_rows = executor::block_on(sql::query_stock_list(&columns, " where market in ('主板', '中小板')")).unwrap();
        let mut count = 0;
        let mut each_thread_codes = Vec::<String>::with_capacity(EACH_THREAD_FETCH_NUM);
        for row in &stock_codes_rows {
            let ts_code: String = row.get("ts_code");
            each_thread_codes.push(ts_code);
            count = count + 1;
            if count == EACH_THREAD_FETCH_NUM {
                println!("thread num!!!!!");
                tokio_runtime.spawn(fetch_index_info(each_thread_codes));
                each_thread_codes = Vec::<String>::with_capacity(EACH_THREAD_FETCH_NUM);
                count = 0;
            }
        }

        self.is_started = true;
    }
}