use chrono::{Local, DateTime, Duration};
use std::ops::Sub;

pub fn curr_date() -> DateTime<Local> {
    Local::now()
}

pub fn curr_date_str(format: &str) -> String {
    let date_time = Local::now();
    date_time.format("%Y%m%d").to_string()
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