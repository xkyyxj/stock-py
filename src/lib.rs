mod py_wrapper;
mod initialize;
mod time;
mod sql;
mod config;
mod results;
mod calculate;
mod cache;
mod analyzer;
mod utils;

use pyo3::prelude::*;
use pyo3::{wrap_pyfunction};
use initialize::init;
use calculate::calculate_in_low;
use pyo3::types::PyDict;
use std::collections::HashMap;
use futures::executor;

#[pymodule]
fn stock_py(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(initialize)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_in_low_sync)).unwrap();
    module.add_wrapped(wrap_pyfunction!(calculate_in_low_async)).unwrap();
    module.add_class::<py_wrapper::TimeFetcher>()?;
    Ok(())
}

#[pyfunction(kwds="**")]
fn initialize(kwds: Option<&PyDict>) -> PyResult<String> {
    let real_param = kwds.unwrap();
    if real_param.len() < 2 {
        return Err(PyErr::new::<pyo3::exceptions::Exception, _>("Error message"));
    }

    let mut para_map = HashMap::<String, String>::with_capacity(2);
    let mysql_info = real_param.get_item("mysql").unwrap().to_string();
    let redis_info = real_param.get_item("redis").unwrap().to_string();
    para_map.insert(String::from("mysql"), mysql_info);
    para_map.insert(String::from("redis"), redis_info);
    initialize::init(para_map);
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
fn calculate_in_low_sync(kwds: Option<&PyDict>) -> PyResult<String> {
    executor::block_on(calculate_in_low());
    Ok(String::from("finished"))
}

#[pyfunction(kwds="**")]
fn calculate_in_low_async(kwds: Option<&PyDict>) -> PyResult<String> {
    let runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
    runtime.spawn(calculate_in_low());
    Ok(String::from("finished"))
}