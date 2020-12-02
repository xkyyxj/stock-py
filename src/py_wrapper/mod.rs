mod time_fetcher;
mod common_calculate;
mod analyzer;
mod common_select;

pub use time_fetcher::TimeFetcher;
pub use analyzer::HistoryDownAna;
pub use common_select::CommonSelectStrategy;

use pyo3::prelude::PyModule;
use common_calculate::init_common_calculate;

pub fn init_py_module(module: &PyModule) {
    init_common_calculate(module);
}