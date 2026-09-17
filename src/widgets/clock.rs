use chrono::{DateTime, Local};

pub fn current_time() -> DateTime<Local> {
    Local::now()
}

pub fn formatted_time() -> String {
    let now = current_time();
    let formatted = format!("{}", now.format("%H:%M"));
    formatted
}
