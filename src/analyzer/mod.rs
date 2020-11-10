mod index_analyzer;
mod history_down_ana;
mod box_style_ana;

pub use history_down_ana::HistoryDownAnalyzer;
use chrono::{Local, TimeZone, DateTime, Duration, Datelike};
use async_std::task::sleep;

pub struct SleepDuringStop {
    // 当前天上午开盘时间(上午9:29:59)
    _up_begin_time: DateTime<Local>,
    // 当前天上午闭盘时间(上午11:59:59)
    _up_end_time: DateTime<Local>,
    // 当前天下午开盘时间(上午12:59:59)
    _down_begin_time: DateTime<Local>,
    // 当前天下午闭盘时间(上午14:59:59)
    _down_end_time: DateTime<Local>,
}

impl SleepDuringStop {
    pub(crate) fn new() -> Self {
        let local: DateTime<Local> = Local::now();
        let year = local.date().year();
        let month = local.date().month();
        let day = local.date().day();
        SleepDuringStop{
            // 当前天上午开盘时间(上午9:29:59)
            _up_begin_time: Local.ymd(year, month, day).and_hms_milli(9, 29, 59, 0),
            // 当前天上午闭盘时间(上午11:59:59)
            _up_end_time: Local.ymd(year, month, day).and_hms_milli(11, 59, 59, 0),
            // 当前天下午开盘时间(上午12:59:59)
            _down_begin_time: Local.ymd(year, month, day).and_hms_milli(12, 59, 59, 0),
            // 当前天下午闭盘时间(上午14:59:59)
            _down_end_time: Local.ymd(year, month, day).and_hms_milli(14, 59, 59, 0),
        }
    }

    pub(crate) async fn check_sleep(&mut self, curr_time: &DateTime<Local>) {
        // let curr_time: DateTime<Local> = Local::now();
        if (curr_time >= &self._up_begin_time && curr_time <= &self._up_end_time) ||
            (curr_time >= &self._down_begin_time && curr_time <= &self._down_end_time) {
            return;
        }
        else {
            if curr_time > &self._up_end_time && curr_time <= &self._down_begin_time {
                let temp_duration = (self._down_begin_time - self._up_end_time).to_std().unwrap();
                sleep(temp_duration).await;
            }

            // 到了第二天，重新计时吧
            if curr_time > &self._down_end_time {
                let next_day_duration = Duration::hours(24);
                self._up_begin_time = self._up_begin_time + next_day_duration;
                let temp_duration = (self._up_begin_time - self._down_end_time).to_std().unwrap();
                self._up_end_time = self._up_end_time + next_day_duration;
                self._down_begin_time = self._down_begin_time + next_day_duration;
                self._down_end_time = self._down_end_time + next_day_duration;
                sleep(temp_duration).await;
            }
        }
    }
}