use pyo3::prelude::*;
use async_std::task;
use crate::sold::TrackSold;

#[pyclass]
pub struct TrackSoldStrategy {
    pub(crate) is_started: bool,
}

#[pymethods]
impl TrackSoldStrategy {
    #[new]
    pub(crate) fn new() -> Self {
        TrackSold {
            is_started: false
        }
    }

    #[call]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }
        task::spawn(async {
            let mut select = TrackSold::new().await;
            select.initialize().await;
            select.select().await;
        });
        self.is_started = true;
    }
}