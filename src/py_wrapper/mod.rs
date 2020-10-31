mod time_fetcher;
mod common_calculate;

pub use time_fetcher::TimeFetcher;

use pyo3::prelude::PyModule;
use common_calculate::init_common_calculate;

pub fn init_py_module(module: &PyModule) {
    init_common_calculate(module);
}