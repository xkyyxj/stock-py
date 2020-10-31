use pyo3::{wrap_pyfunction};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use futures::executor;
use crate::calculate::calculate_in_low;
use crate::calculate::calculate_history_down;

pub fn init_common_calculate(module: &PyModule) {
    module.add_wrapped(wrap_pyfunction!(calculate_in_low_sync)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_in_low_async)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_history_down_async)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_history_down_sync)).unwrap();
}

#[pyfunction(kwds="**")]
pub fn calculate_in_low_sync(kwds: Option<&PyDict>) -> PyResult<String> {
    executor::block_on(calculate_in_low());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_in_low_async(kwds: Option<&PyDict>) -> PyResult<String> {
    let runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    runtime.spawn(calculate_in_low());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_history_down_sync(kwds: Option<&PyDict>) -> PyResult<String> {
    executor::block_on(calculate_history_down());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_history_down_async(kwds: Option<&PyDict>) -> PyResult<String> {
    let runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    runtime.spawn(calculate_history_down());
    Ok(String::from("finished"))
}