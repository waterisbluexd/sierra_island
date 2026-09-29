use smithay_client_toolkit::reexports::calloop::channel::channel;

use crate::wayland::state::SierraState;

pub mod notify;
pub use notify::{apply_theme, start_watcher};

pub fn install_watcher(
    loop_handle: &smithay_client_toolkit::reexports::calloop::LoopHandle<'_, SierraState>,
) {
    let (theme_sender, theme_channel) = channel::<()>();

    start_watcher(theme_sender);

    let _theme_lh = loop_handle.clone();
    loop_handle
        .insert_source(theme_channel, move |event, _, state| {
            if let smithay_client_toolkit::reexports::calloop::channel::Event::Msg(()) = event {
                let theme = crate::theme::Theme::load();

                apply_theme(&state.island, &theme);

                slint::platform::update_timers_and_animations();
                let _ = state.draw();
            }
        })
        .expect("Failed to insert theme watcher");
}
