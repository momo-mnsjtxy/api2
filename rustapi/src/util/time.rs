use chrono::{Datelike, Local, TimeZone};

pub fn range_today() -> (i64, i64) {
    let now = Local::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let start = Local.from_local_datetime(&start).unwrap().timestamp();
    (start, start + 86400 - 1)
}

pub fn range_yesterday() -> (i64, i64) {
    let (t0, _) = range_today();
    (t0 - 86400, t0 - 1)
}

pub fn range_this_week() -> (i64, i64) {
    let now = Local::now();
    let weekday = now.weekday().num_days_from_monday() as i64;
    let (today, _) = range_today();
    let start = today - weekday * 86400;
    (start, start + 7 * 86400 - 1)
}

pub fn range_last_week() -> (i64, i64) {
    let (start, _) = range_this_week();
    (start - 7 * 86400, start - 1)
}

pub fn range_this_month() -> (i64, i64) {
    let now = Local::now();
    let start = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start_ts = Local.from_local_datetime(&start).unwrap().timestamp();
    let next = if now.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
    }
    .and_hms_opt(0, 0, 0)
    .unwrap();
    let end = Local.from_local_datetime(&next).unwrap().timestamp() - 1;
    (start_ts, end)
}

pub fn range_last_month() -> (i64, i64) {
    let now = Local::now();
    let (y, m) = if now.month() == 1 {
        (now.year() - 1, 12)
    } else {
        (now.year(), now.month() - 1)
    };
    let start = chrono::NaiveDate::from_ymd_opt(y, m, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let start_ts = Local.from_local_datetime(&start).unwrap().timestamp();
    let (this_start, _) = range_this_month();
    (start_ts, this_start - 1)
}

pub fn day_range_offset(days_ago: i64) -> (i64, i64) {
    let (today, _) = range_today();
    let start = today - days_ago * 86400;
    (start, start + 86399)
}

pub fn now_ts() -> i64 {
    Local::now().timestamp()
}
