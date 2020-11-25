use chrono::{Local, DateTime, Duration, Datelike, TimeZone};
use std::ops::Sub;
use async_std::task::sleep;

pub struct SleepDuringStop {
    // 当前天上午开盘时间(上午9:29:59)
    _up_begin_time: DateTime<Local>,
    // 当前天上午闭盘时间(上午11:29:59)
    _up_end_time: DateTime<Local>,
    // 当前天下午开盘时间(上午12:59:59)
    _down_begin_time: DateTime<Local>,
    // 当前天下午闭盘时间(上午14:59:59)
    _down_end_time: DateTime<Local>,
}

impl SleepDuringStop {
    pub fn new() -> Self {
        let local: DateTime<Local> = Local::now();
        let year = local.date().year();
        let month = local.date().month();
        let day = local.date().day();
        SleepDuringStop{
            // 当前天上午开盘时间(上午9:29:59)
            _up_begin_time: Local.ymd(year, month, day).and_hms_milli(9, 29, 59, 0),
            // 当前天上午闭盘时间(上午11:29:59)
            _up_end_time: Local.ymd(year, month, day).and_hms_milli(11, 29, 59, 0),
            // 当前天下午开盘时间(上午12:59:59)
            _down_begin_time: Local.ymd(year, month, day).and_hms_milli(12, 59, 59, 0),
            // 当前天下午闭盘时间(上午14:59:59)
            _down_end_time: Local.ymd(year, month, day).and_hms_milli(14, 59, 59, 0),
        }
    }

    pub async fn check_sleep(&mut self, curr_time: &DateTime<Local>) {
        // let curr_time: DateTime<Local> = Local::now();
        if (curr_time >= &self._up_begin_time && curr_time <= &self._up_end_time) ||
            (curr_time >= &self._down_begin_time && curr_time <= &self._down_end_time) {
            return;
        }
        else {
            // 早上未开盘之前，休眠
            if curr_time < &self._up_begin_time {
                let temp_duration = (self._up_begin_time - *curr_time).to_std().unwrap();
                sleep(temp_duration).await;
            }

            // 午休休盘时间，睡眠
            if curr_time > &self._up_end_time && curr_time <= &self._down_begin_time {
                let temp_duration = (self._down_begin_time - *curr_time).to_std().unwrap();
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

    /// 查询调用方法的传入时间是不是夜间：
    /// 当天早盘开盘之钱或者下午闭市之后 -> 返回true
    /// 其他时间点 -> 返回false
    pub fn check_curr_night_rest(&self, curr_time: &DateTime<Local>) -> bool {
        curr_time > &self._down_end_time || curr_time < &self._up_begin_time
    }
}

pub fn curr_date() -> DateTime<Local> {
    Local::now()
}

pub fn curr_date_str(format: &str) -> String {
    let date_time = Local::now();
    date_time.format(format).to_string()
}

pub fn curr_date_before_days(days: i64) -> DateTime<Local> {
    let mut date_time = Local::now();
    let duration = Duration::days(days);
    date_time.sub(duration)
}

pub fn curr_date_before_days_str(days: i64, format: &str) -> String {
    let date_time = curr_date_before_days(days);
    date_time.format(format).to_string()
}