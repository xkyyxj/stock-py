use crate::results::TimeIndexBaseInfo;
use crate::selector::ema_select::{EMASelect};
use futures::{Future, executor};
use std::pin::Pin;
use crate::utils::time_utils::SleepDuringStop;
use chrono::{Local, Duration};
use std::sync::mpsc;
use crate::selector::SelectResult;
use async_std::task::sleep;

pub struct ShortTimeSelect {
    ema_select: EMASelect,
    sleep_check: SleepDuringStop,
}

impl ShortTimeSelect {
    pub(crate) async fn new() -> Self {
        let mut ret_val = ShortTimeSelect {
            ema_select: EMASelect::new().await,
            sleep_check: SleepDuringStop::new()
        };
        ret_val
    }

    pub async fn initialize(&mut self) {
        self.ema_select.initialize().await;
    }

    pub(crate) async fn select(&mut self) {
        // 第零步：获取初始化的配置信息
        let config = crate::initialize::CONFIG_INFO.get().unwrap();
        let ana_delta_time = config.analyze_time_delta;
        let taskbar = crate::initialize::TASKBAR_TOOL.get().unwrap();
        let tokio_runtime = crate::initialize::TOKIO_RUNTIME.get().unwrap();
        loop {
            let (tx, rx) = mpsc::channel::<SelectResult>();
            let curr_time = Local::now();
            self.sleep_check.check_sleep(&curr_time).await;
            let tx2 = tx.clone();
            let future = self.ema_select.select(tx2);
            // let future2 = self.ema_select.select(tx);
            futures::join!(future);
            // tokio_runtime.spawn(future);
            for received  in rx {
                //
            }

            // 每两秒获取一次
            let two_seconds_duration = Duration::seconds(ana_delta_time);
            let fetch_finish_time = Local::now();
            let fetch_cost_time = fetch_finish_time - curr_time;
            let real_sleep_time = two_seconds_duration - fetch_cost_time;
            if real_sleep_time.num_nanoseconds().unwrap() > 0 {
                sleep(real_sleep_time.to_std().unwrap()).await;
            }
            // 任务栏弹出提示通知消息(评分大于等于60就买入吧)
            // if wait_select_stock.len() > 0 {
            //     println!("EMA信号");
            //     taskbar.show_win_toast(String::from("EMA Select:"), wait_select_stock);
            // }
        }
    }

    /// 处理策略：如果是
    fn process_ana_result(&mut self, result: SelectResult) {

    }
}