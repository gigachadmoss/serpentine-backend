pub fn current_time_secs() -> i64 {
    let now = chrono::Utc::now();

    now.timestamp()
}

pub fn current_time_micros() -> i64 {
    let now = chrono::Utc::now();

    now.timestamp_micros()
}
