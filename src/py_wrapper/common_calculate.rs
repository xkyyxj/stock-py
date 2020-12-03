use pyo3::{wrap_pyfunction};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use futures::executor;
use async_std::task;
use crate::calculate::{ calculate_air_castle, calculate_all };
use crate::calculate::calculate_history_down;
use crate::py_wrapper::common_select::CommonSelectStrategy;

pub fn init_common_calculate(module: &PyModule) {
    module.add_class::<CommonSelectStrategy>().unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_all_sync)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_all_async)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_history_down_async)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_history_down_sync)).unwrap();
}

#[pyfunction(kwds="**")]
pub fn calculate_all_sync(_kwds: Option<&PyDict>) -> PyResult<String> {
    task::block_on(calculate_all());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_all_async(_kwds: Option<&PyDict>) -> PyResult<String> {
    task::spawn(calculate_all());
    Ok(String::from("finished"))
}


#[pyfunction(kwds="**")]
pub fn calculate_history_down_sync(_kwds: Option<&PyDict>) -> PyResult<String> {
    task::block_on(calculate_history_down());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_history_down_async(_kwds: Option<&PyDict>) -> PyResult<String> {
    task::spawn(calculate_history_down());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
pub fn calculate_air_castle_async(_kwds: Option<&PyDict>) -> PyResult<String> {
    task::spawn(calculate_air_castle());
    Ok(String::from("finished"))
}
