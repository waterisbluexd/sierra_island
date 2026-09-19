use chrono::{Datelike, Local, Timelike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockData {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,

    pub day: u32,
    pub month: u32,
    pub year: i32,

    pub weekday: u32,
}

impl ClockData {
    pub fn now() -> Self {
        let now = Local::now();

        Self {
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),

            day: now.day(),
            month: now.month(),
            year: now.year(),

            weekday: now.weekday().num_days_from_monday(),
        }
    }
}
