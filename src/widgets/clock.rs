use chrono::{Local, Timelike};

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
