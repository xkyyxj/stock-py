use crate::analyzer::HistoryDownAnalyzer;

use pyo3::prelude::*;



use async_std::task;




async fn history_down_wrapper() {
    let mut real_analyzer = HistoryDownAnalyzer::new();
    real_analyzer.analyze().await;
}

#[pyclass]
pub struct HistoryDownAna {
    pub(crate) is_started: bool,
}

#[pymethods]
impl HistoryDownAna {
    #[new]
    fn new() -> Self {
        HistoryDownAna {
            is_started: false
        }
    }

    #[call]
    #[args(args="*")]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }
        task::spawn(history_down_wrapper());
        self.is_started = true;
    }
}