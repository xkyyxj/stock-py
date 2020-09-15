mod pywrapper;

use pyo3::prelude::*;
use pyo3::{wrap_pyfunction};
use initialize::init;

#[pymodule]
fn stock_rust(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<TimesOperator>()?;
    module.add_wrapped(wrap_pyfunction!(initialize)).unwrap();
    Ok(())
}

#[pyfunction]
fn initialize() {

}

#[pyclass]
struct TimesOperator {}

impl TimesOperator {

}