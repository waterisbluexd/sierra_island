use std::time::Duration;

use chrono::Datelike;
use chrono::Timelike;
use slint::ComponentHandle;

use smithay_client_toolkit::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::wayland::state::SierraState;

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

pub fn install_clock_timer(
    loop_handle: &smithay_client_toolkit::reexports::calloop::LoopHandle<'_, SierraState>,
) {
    let _clock_lh = loop_handle.clone();
    loop_handle
        .insert_source(
            Timer::from_duration(Duration::from_secs(1)),
            move |_, _, state| {
                state.update_time();

                slint::platform::update_timers_and_animations();
                let _ = state.draw();

                TimeoutAction::ToDuration(Duration::from_secs(1))
            },
        )
        .expect("Failed to insert clock timer");
}
