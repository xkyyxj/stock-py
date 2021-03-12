use chrono::{Local, DateTime, Duration, Datelike, TimeZone};
use std::ops::Sub;
use async_std::task::sleep;
use async_std::sync::Mutex;
use std::collections::HashMap;
use std::task::Waker;
use std::future::Future;
use winapi::_core::task::{Context, Poll};
use winapi::_core::pin::Pin;
use std::{thread, time};
use async_std::task;
use log::{error, info, warn};

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
                let temp_duration = self._up_begin_time - *curr_time;
                // sleep(temp_duration).await;
                my_sleep(temp_duration).await;
                // small_step_sleep(&temp_duration).await;
            }

            // 午休休盘时间，睡眠
            if curr_time > &self._up_end_time && curr_time <= &self._down_begin_time {
                let temp_duration = self._down_begin_time - *curr_time;
                // sleep(temp_duration).await;
                my_sleep(temp_duration).await;
                // small_step_sleep(&temp_duration).await;
            }

            // 到了第二天，重新计时吧
            if curr_time > &self._down_end_time {
                let next_day_duration = Duration::hours(24);
                self._up_begin_time = self._up_begin_time + next_day_duration;
                let temp_duration = self._up_begin_time - self._down_end_time;
                self._up_end_time = self._up_end_time + next_day_duration;
                self._down_begin_time = self._down_begin_time + next_day_duration;
                self._down_end_time = self._down_end_time + next_day_duration;
                // sleep(temp_duration).await;
                my_sleep(temp_duration).await;
                // small_step_sleep(&temp_duration).await;
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

pub async fn my_sleep(duration: Duration) {
    error!("is running?????");
    SleepUnit::new(duration).await
}

pub async fn small_step_sleep(duration: &Duration) {
    let mut temp_duration = duration.clone();
    // 每隔三秒睡眠一段时间（为了处理windows这个非实时操作系统）
    while temp_duration.num_seconds() > 3 {
        let three_duration = Duration::seconds(3);
        sleep(three_duration.clone().to_std().unwrap()).await;
        temp_duration = (temp_duration - three_duration);
        println!("cur seconds is {}", temp_duration.num_seconds());
    }
    if temp_duration.num_seconds() > 0 {
        sleep(temp_duration.to_std().unwrap()).await;
    }
}

pub fn curr_date() -> DateTime<Local> {
    Local::now()
}

/// 通用模式："%Y%m%d"，备忘一下，全的：%Y-%m-%d %H:%M:%S
pub fn curr_date_str(format: &str) -> String {
    let date_time = Local::now();
    date_time.format(format).to_string()
}

pub fn curr_date_before_days(days: i64) -> DateTime<Local> {
    let date_time = Local::now();
    let duration = Duration::days(days);
    date_time.sub(duration)
}

pub fn curr_date_before_days_str(days: i64, format: &str) -> String {
    let date_time = curr_date_before_days(days);
    date_time.format(format).to_string()
}


pub struct SleepUnit {
    pub start_time: DateTime<Local>,
    pub finish_time: DateTime<Local>
}

impl SleepUnit {
    pub fn new(duration: Duration) -> Self {
        let curr_time = Local::now();
        SleepUnit {
            start_time: Local::now(),
            finish_time: curr_time + duration
        }
    }
}

impl Future for SleepUnit {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // error!("is polling??");
        let curr_time = Local::now();
        return if curr_time < self.finish_time {
            // error!("not finished!!");
            let temp_waker = cx.waker().clone();
            let time_check = crate::initialize::TIME_CHECK.get().unwrap();
            time_check.add_time(self.finish_time.clone(), temp_waker);
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}


/// 为了应对windows下面的情况，非实时系统
pub struct TimeCheck {
    time_wake: Mutex<HashMap::<DateTime<Local>,Vec::<Waker>>>
}

impl TimeCheck {

    pub fn new() -> Self {
        TimeCheck {
            time_wake: Mutex::new(HashMap::<DateTime<Local>,Vec::<Waker>>::new()),
        }
    }

    pub fn add_time(&self, time: DateTime<Local>, waker: Waker) {
        let mut map_guard = task::block_on(async {
            self.time_wake.lock().await
        });
        let temp = (*map_guard).get_mut(&time);
        match temp {
            None => {
                let mut temp_vec = vec![];
                temp_vec.push(waker);
                // error!("add finished!");
                map_guard.insert(time, temp_vec);
            }
            Some(val) => {
                // error!("add finished!222222");
                val.push(waker);
            }
        }
    }

    pub fn start(&self) {
        loop {
            let mut map_guard = task::block_on(async {
                self.time_wake.lock().await
            });
            let curr_time = Local::now();
            if map_guard.is_empty() {
                drop(map_guard);
                thread::sleep(time::Duration::from_secs(2));
                continue;
            }

            let mut keys = map_guard.keys();
            let mut keys_vec = vec![];
            for key in keys {
                keys_vec.push(key.clone());
            }

            for key in keys_vec {
                // error!("time is {}， curr time is {}", key, curr_time);
                if key > curr_time {
                    continue;
                }
                // error!("wake1");
                let waker = map_guard.get_mut(&key).unwrap();
                loop {
                    if waker.len() <= 0 {
                        // error!("waker length is 0");
                        break;
                    }
                    // error!("wake2");
                    let temp_waker = waker.pop().unwrap();
                    temp_waker.wake();
                }

                if map_guard.get(&key).unwrap().len() == 0 {
                    map_guard.remove(&key);
                }
            }
            drop(map_guard);
            thread::sleep(time::Duration::from_secs(2));
            thread::park_timeout();
        }
    }
}