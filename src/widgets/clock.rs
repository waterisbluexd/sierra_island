use chrono::Timelike;

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
