use crate::selector::{CommonSelectRst, FINAL_TYPE};
use crate::file::read_txt_file;
use std::collections::HashMap;
use crate::selector::ema_select::EMASelect;
use crate::selector::history_down_select::HistoryDownSelect;
use futures::{StreamExt};
use crate::selector::rst_process::CommonTimeRstProcess;
use crate::utils::time_utils::SleepDuringStop;
use chrono::Local;
use async_std::task::{self, sleep};
use futures::channel::mpsc;
use chrono::Duration;
use log::{error, info, warn, trace};


// async_std的MutexGuard是实现了Send的，标准库的MutexGuard则没有（因为task对象可能在不同的线程上切换然后执行）
// 否则的话你说两个future如何并行执行？？？？？？？？？？？？？（按照现在这写法）
use async_std::sync::{Mutex, Arc};
use crate::simulate::BuyInfoInsert;


pub struct AllSelectStrategy {
    rst_processor: CommonTimeRstProcess,
    short_time_selector: Vec<String>,
    long_time_selector: Vec<String>,

    time_check: SleepDuringStop,

    // 各种选择策略：
    ema_select: Arc<Mutex<EMASelect>>,
    history_down: Arc<Mutex<HistoryDownSelect>>,

    insert_simulate: BuyInfoInsert,
}

impl AllSelectStrategy {
    pub async fn new() -> Self {
        AllSelectStrategy {
            rst_processor: CommonTimeRstProcess::new(),
            short_time_selector: vec![],
            long_time_selector: vec![],
            time_check: SleepDuringStop::new(),
            ema_select: Arc::new(Mutex::new(EMASelect::new().await)),
            history_down: Arc::new(Mutex::new(HistoryDownSelect::new().await)),
            insert_simulate: BuyInfoInsert::new(),
        }
    }

    pub async fn initialize(&mut self) {
        let mut infos = parse_file().await;
        if let Some(selectors) = infos.remove("short_time") {
            self.short_time_selector = selectors;
        }
        if let Some(selectors) = infos.remove("long_time") {
            self.long_time_selector = selectors;
        }

        if self.contain_selector(&String::from(EMASelect::get_name())) {
            let real_ema = &mut *self.ema_select.lock().await;
            real_ema.initialize().await;
        }

        if self.contain_selector(&String::from(HistoryDownSelect::get_name())) {
            let real_history_down = &mut *self.history_down.lock().await;
            real_history_down.initialize().await;
        }

        self.insert_simulate.initialize();
        self.rst_processor.initialize().await;
    }

    fn contain_selector(&mut self, name: &String) -> bool {
        self.short_time_selector.contains(name) || self.long_time_selector.contains(name)
    }

    pub async fn select(&mut self) {
        // 第零步：获取初始化的配置信息
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let ana_delta_time = config.analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        loop {
            warn!("Selector loop!");
            let curr_time = Local::now();
            match self.time_check.check_sleep(&curr_time).await {
                crate::utils::time_utils::MIDNIGHT_SLEEP => {
                    // 如果是晚上的话，刷新一下买入信息
                    self.insert_simulate.refresh();
                    (*self.history_down.lock().await).refresh().await;
                    (*self.ema_select.lock().await).refresh().await;
                },
                crate::utils::time_utils::BEFORE_MORNING_SLEEP => {
                    // 早上之前同样需要刷新一下数据
                    self.insert_simulate.refresh();
                    (*self.history_down.lock().await).refresh().await;
                    (*self.ema_select.lock().await).refresh().await;
                },
                _ => {}
            }
            let (tx, rx) = mpsc::unbounded::<CommonSelectRst>();
            // 如果添加了新的选择策略，别忘了在这儿添加，现在只能是这样了…………，动态扩展？？？？？？？呵呵哒哒
            let history_down_clone = self.history_down.clone();
            let mut tx_clone = tx.clone();
            task::spawn(async move {
                let real_history_down = &mut *history_down_clone.lock().await;
                real_history_down.select(tx_clone).await;
            });

            let ema_select_clone = self.ema_select.clone();
            tx_clone = tx.clone();
            task::spawn(async move {
                let real_ema_select = &mut *ema_select_clone.lock().await;
                real_ema_select.select(tx_clone).await;
            });
            drop(tx);

            let mut temp_rst = CommonSelectRst::new();
            let all_common_rst = rx.collect::<Vec<CommonSelectRst>>().await;
            for item in all_common_rst {
                temp_rst.merge(&item);
            }
            self.insert_simulate.write_select_rst_2_simulate(&temp_rst).await;
            // TODO -- 如何选择出最终的wait_select结果？？？？？
            judge_wait_select(&mut temp_rst);
            let new_stocks = self.rst_processor.process(&temp_rst, &curr_time).await;
            let title = String::from("new s");
            let mut content = String::from("");
            for item in new_stocks {
                content = content + item.as_str() + ",";
            }
            taskbar.show_win_toast(title, content);

            // 每X秒获取一次(由analyze_time_delta指定)
            let duration = Duration::seconds(ana_delta_time);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
        }
    }
}

// 辅助函数--解析配置文件
async fn parse_file() -> HashMap<String, Vec<String>> {
    let mut ret_val = HashMap::<String, Vec<String>>::new();
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
        let mut long_or_short = String::from(*infos.get(0).unwrap());
        long_or_short = String::from(long_or_short.trim());
        let mut selectors = String::from(*infos.get(1).unwrap());
        selectors = String::from(selectors.trim());
        if long_or_short == "short_time" || long_or_short == "long_time" {
            ret_val.insert(long_or_short, parse_selectors(selectors).await);
        }
    }
    ret_val
}

async fn parse_selectors(selectors: String) -> Vec<String> {
    let mut all_selectors = Vec::<String>::new();
    let all_selectors_str: Vec<&str> = selectors.split(',').collect();
    for selector in all_selectors_str {
        let mut str = String::from(selector);
        str = String::from(str.trim());
        if str.len() == 0 {
            continue;
        }
        trace!("infos is {}", str);
        all_selectors.push(str);
    }
    all_selectors
}

/// 票选待选
/// 根据config当中指定的，将评分最高的几只股票放到wait_select当中
fn judge_wait_select(rst: &mut CommonSelectRst) {
    // 按照从大到小的顺序排列
    rst.select_rst.sort_by(|a, b| {
       b.level.cmp(&a.level)
    });

    // 选出多少只股票来，由Config来判定
    let config = &crate::initialize::CONFIG_INFO.get().unwrap().wait_select_config;
    let num = config.max_wait_select_each_day;

    for i in 0..rst.select_rst.len() {
        if let Some(rst) = rst.select_rst.get_mut(i as usize) {
            if i < num as usize {
                rst.rst_style = rst.rst_style | FINAL_TYPE;
            }
            else {
                rst.rst_style = rst.rst_style & !FINAL_TYPE;
            }
        }
    }
}