use std::time::{Duration, Instant};

use smithay_client_toolkit::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::wayland::state::SierraState;

pub fn install_animation_timer(
    loop_handle: &smithay_client_toolkit::reexports::calloop::LoopHandle<'_, SierraState>,
) {
    let _animation_lh = loop_handle.clone();
    loop_handle
        .insert_source(
            Timer::from_duration(Duration::from_millis(16)),
            move |_instant, _, state| {
                state.update_hover(Instant::now());

                slint::platform::update_timers_and_animations();
                let _ = state.draw();

                let interval_ns =
                    1_000_000_000_000 / state.refresh_rate_mhz.max(1000) as u64;
                TimeoutAction::ToDuration(Duration::from_nanos(interval_ns))
            },
        )
        .expect("Failed to insert animation timer");
}
