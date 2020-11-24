mod common_simulate_rst;
mod i_short_history;

pub use i_short_history::sync_short_history;

pub struct Simulation {

}

impl Simulation {
    pub(crate) fn new() -> Self {
        Simulation {}
    }

    pub(crate) async fn simulate(&self) {

    }
}