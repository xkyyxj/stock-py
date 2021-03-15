use pyo3::prelude::*;
use async_std::task;
use crate::sold::TrackSold;

pub fn init_track_sold(module: &PyModule) {
    module.add_class::<TrackSoldStrategy>().unwrap();
}

#[pyclass]
pub struct TrackSoldStrategy {
    pub(crate) is_started: bool,
}

#[pymethods]
impl TrackSoldStrategy {
    #[new]
    pub(crate) fn new() -> Self {
        TrackSoldStrategy {
            is_started: false
        }
    }

    #[call]
    pub(crate) fn __call__(&mut self){
        if self.is_started {
            return
        }
        task::spawn(async {
            let mut sold = TrackSold::new();
            sold.initialize().await;
            sold.sold().await;
        });
        self.is_started = true;
    }
}