mod time_fetcher;
mod common_calculate;
mod analyzer;
mod common_select;
mod track_sold;

pub use time_fetcher::TimeFetcher;
pub use analyzer::HistoryDownAna;
pub use common_select::CommonSelectStrategy;

use pyo3::prelude::PyModule;
use common_calculate::init_common_calculate;
use crate::py_wrapper::track_sold::init_track_sold;

pub fn init_py_module(module: &PyModule) {
    init_common_calculate(module);
    init_track_sold(module);
}