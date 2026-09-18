use chrono::{DateTime, Local, Timelike};

pub fn current_time() -> DateTime<Local> {
    Local::now()
}

pub fn current_hour() -> u32 {
    let now = current_time();
    now.hour()
}

pub fn current_minute() -> u32 {
    let now = current_time();
    now.minute()
}
