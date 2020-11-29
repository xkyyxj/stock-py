use crate::selector::long_time_select::{LongTimeSelectResult, LongTimeSelect};
use crate::selector::{ShortTimeSelect, CommonSelectRst};
use crate::file::read_txt_file;
use std::collections::HashMap;
use crate::selector::ema_select::EMASelect;
use crate::selector::history_down_select::HistoryDownSelect;
use std::sync::mpsc;

pub struct AllSelectStrategy {
    short_time: ShortTimeSelect,
    long_time: LongTimeSelect,
    short_time_selector: Vec<String>,
    long_time_selector: Vec<String>,
    selector_rst: HashMap<String, Vec::<CommonSelectRst>>,

    // 各种选择器
    ema_select: EMASelect,
    history_down: HistoryDownSelect,
}

impl AllSelectStrategy {
    pub fn new() -> Self {

    }

    pub async fn initialize(&mut self) {
        let mut infos = parse_file().await;
        if let Some(selectors) = infos.remove("short_time") {
            self.short_time_selector = selectors;
        }
        if let Some(selectors) = infos.remove("long_time") {
            self.long_time_selector = selectors;
        }
    }

    pub async fn select(&self) {
        // 第零步：获取初始化的配置信息
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let ana_delta_time = config.analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        loop {
            let (tx, rx) = mpsc::channel::<CommonSelectRst>();
        }
    }
}

// 辅助函数--解析配置文件
async fn parse_file() -> HashMap<String, Vec<String>> {
    let ret_val = HashMap::<String, Vec<String>>::new();
    let content = read_txt_file(String::from("select_config")).await;
    let all_str_rows: Vec<&str> = content.split('\n').collect();
    for row in all_str_rows {
        let row_str = String::from(row);
        // 忽略注释行以及空行
        if row_str.starts_with('#') || row_str.trim().len() == 0 {
            continue;
        }

        // 解析
        let infos: Vec<&str> = row_str.split('=').collect();
        if infos.len() < 2 {
            continue;
        }
        let mut long_or_short = String::from(infos.get(0).unwrap());
        long_or_short = String::from(long_or_short.trim());
        let mut selectors = String::from(infos.get(1).unwrap());
        selectors = String::from(selectors.trim());
        if long_or_short == "short_time" || long_or_short == "long_time" {
            ret_val.put(long_or_short, parse_selectors(selectors));
        }
    }
    ret_val
}

async fn parse_selectors(selectors: String) -> Vec<String> {
    let mut all_selectors = Vec::<String>::new();
    let all_selectors_str: Vec<&str> = selectors.split(',').trim();
    for selector in all_selectors {
        let mut str = String::from(selector);
        str = String::from(str.trim());
        if str.len() == 0 {
            continue;
        }
        all_selectors.push(str);
    }
    all_selectors
}