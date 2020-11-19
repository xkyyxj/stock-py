mod ema_select;
mod short_time_select;

pub use short_time_select::ShortTimeSelect;
use chrono::{DateTime, Local};

pub(crate) struct SingleSelectResult {
    pub(crate) ts_code: String,
    pub(crate) ts_name: String,
    pub(crate) level: i64,              // 评分：0-100分
    pub(crate) source: String,          // 来源系统，通过ema选定还是什么其他指标
    pub(crate) level_pct: f64,          // 得分的百分比
}

impl SingleSelectResult {
    pub(crate) fn new() -> Self {
        SingleSelectResult {
            ts_code: "".to_string(),
            ts_name: "".to_string(),
            level: 0,
            source: "".to_string(),
            level_pct: 0.0
        }
    }
}

pub(crate) struct SelectResult {
    pub(crate) select_rst: Vec<SingleSelectResult>,
    pub(crate) ts: DateTime<Local>,
}

impl SelectResult {

    pub(crate) fn new() -> Self {
        SelectResult { select_rst: vec![], ts: Local::now() }
    }

    pub(crate) fn add_selected(&mut self, info :SingleSelectResult) {
        self.select_rst.push(info);
    }

    /// 两个结果的合并，得分的简单相加
    pub(crate) fn merge(&mut self, other: &SelectResult) {
        for self_item in &mut self.select_rst {
            for other_item in other.select_rst {
                if self_item.ts_code == other_item.ts_code {
                    self_item.level = self_item.level + other_item.level;
                    if self_item.level > 100 {
                        self_item.level = 100;
                    }
                }
            }
        }
        self.ts = Local::now();
    }
}