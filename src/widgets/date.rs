#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DateFormat {
    DayMonth,
    MonthDay,
    DayMonthWeekday,
    WeekdayDayMonth,
    MonthDayWeekday,
    DayMonthYear,
    DayMonthFullYear,
    DayMonthYearDashed,
    YearMonthDay,
}

impl DateFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DayMonth => "DD MMM",
            Self::MonthDay => "MMM DD",
            Self::DayMonthWeekday => "DD MMM DDD",
            Self::WeekdayDayMonth => "DDD DD MMM",
            Self::MonthDayWeekday => "MMM DD DDD",
            Self::DayMonthYear => "DD/MM/YY",
            Self::DayMonthFullYear => "DD/MM/YYYY",
            Self::DayMonthYearDashed => "DD-MM-YY",
            Self::YearMonthDay => "YYYY-MM-DD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateConfig {
    pub format: DateFormat,
}

impl Default for DateConfig {
    fn default() -> Self {
        Self {
            format: DateFormat::DayMonth,
        }
    }
}
