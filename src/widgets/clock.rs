use chrono::{Local, Timelike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeFormat {
    TwelveHour,
    TwentyFourHour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockConfig {
    pub format: TimeFormat,
    pub show_seconds: bool,
    pub show_am_pm: bool,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: TimeFormat::TwentyFourHour,
            show_seconds: false,
            show_am_pm: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Clock {
    pub fn now() -> Self {
        let now = Local::now();

        Self {
            hour: now.hour(),
            minute: now.minute(),
            second: now.second(),
        }
    }
}
