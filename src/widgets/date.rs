use chrono::{Datelike, Local};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub day: u32,
    pub month: u32,
    pub year: i32,
    pub weekday: u32,
}

impl Date {
    pub fn now() -> Self {
        let now = Local::now();

        Self {
            day: now.day(),
            month: now.month(),
            year: now.year(),
            weekday: now.weekday().num_days_from_monday(),
        }
    }
}
