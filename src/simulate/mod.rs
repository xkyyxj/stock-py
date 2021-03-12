use crate::selector::CommonSelectRst;

mod common_simulate_rst;

pub struct Simulation {

}

impl Simulation {
    pub(crate) fn new() -> Self {
        Simulation {}
    }

    pub(crate) async fn simulate(&self) {

    }
}

/// 将选择结果写入到ope_simulate当中
pub async fn write_select_rst_2_simulate(rst: &CommonSelectRst) {
    if rst.select_rst.is_empty() {
        return;
    }

    for item in &rst.select_rst {

    }
}