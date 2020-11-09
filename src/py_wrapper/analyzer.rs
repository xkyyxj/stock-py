use crate::analyzer::HistoryDownAnalyzer;
use pyo3::types::{PyFunction, PyTuple};
use pyo3::prelude::*;
use std::rc::Rc;
use async_std::sync::Arc;
use std::future::Future;
use futures::channel::mpsc;
use futures::channel::mpsc::Sender;
use futures::StreamExt;

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
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        tokio_runtime.spawn(history_down_wrapper());
        self.is_started = true;
    }
}