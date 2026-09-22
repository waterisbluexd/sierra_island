use slint::platform::PointerEventButton;
use slint::ComponentHandle;
use chrono::Timelike;
use chrono::Datelike;

pub fn update_time_state(island: &crate::Island) {
    let now = chrono::Local::now();

    let clock = island.global::<crate::ClockState>();

    clock.set_hour(now.hour() as i32);

    clock.set_minute(now.minute() as i32);

    clock.set_seconds(now.second() as i32);

    let date = island.global::<crate::DateState>();

    date.set_day(now.day() as i32);

    date.set_month(now.month() as i32);

    date.set_year(now.year());

    date.set_weekday(now.weekday().num_days_from_monday() as i32);

    island.window().request_redraw();
}

pub fn button_from_linux(button: u32) -> PointerEventButton {
    match button {
        0x110 => PointerEventButton::Left,
        0x111 => PointerEventButton::Right,
        0x112 => PointerEventButton::Middle,
        0x113 => PointerEventButton::Back,
        0x114 => PointerEventButton::Forward,
        _ => PointerEventButton::Other,
    }
}
