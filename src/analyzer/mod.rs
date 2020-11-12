mod index_analyzer;
mod history_down_ana;
mod box_style_ana;

pub use history_down_ana::HistoryDownAnalyzer;
use chrono::{Local, TimeZone, DateTime, Duration, Datelike};
use async_std::task::sleep;
use crate::results::TimeIndexBaseInfo;
use crate::time::INDEX_SUFFIX;
use crate::cache::AsyncRedisOperation;

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
    pub(crate) fn new() -> Self {
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

pub async fn get_last_index_info_from_redis(redis_ope: &mut AsyncRedisOperation, ts_code: &String) -> Option<TimeIndexBaseInfo> {
    let mut redis_key = String::from(ts_code) + INDEX_SUFFIX;
    if redis_ope.exists(redis_key).await {
        redis_key = String::from(ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let length = redis_ope.str_length::<String>(redis_key).await;
        // FIXME -- 此处写死了一个值，似乎单条的信息不会超过800吧
        let mut start = length - 800;
        // FIXME -- 此处有一个redis模块(依赖的redis模块，而不是redis服务器)的BUG：如果get_range的start的index正好位于中文字符串的中间，就不能成功返回数据了，此处修正一下
        // FIXME -- 如果start_index小于150，直接到0，这样能够避免这个问题吧，毕竟中文只在开头有
        if start < 150 {
            start = 0;
        }
        redis_key = String::from(ts_code);
        redis_key = redis_key + INDEX_SUFFIX;
        let ret_str = redis_ope.get_range::<String, String>(redis_key, start, length).await.unwrap();
        if ret_str.is_empty() {
            return None;
        }
        // 处理一下字符串，获取到最新的实时信息
        let temp_infos: Vec<&str> = ret_str.split('~').collect();
        if temp_infos.len() < 2 {
            return None;
        }
        if !temp_infos.is_empty() {
            let last_info_str = String::from(*temp_infos.get(temp_infos.len() - 2).unwrap());
            Some(last_info_str.into())
        }
        else {
            None
        }
    }
    else {
        None
    }
}