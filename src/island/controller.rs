use crate::{
    themer,
    widgets::{clock, date},
};

use super::{animation::AnimationState, hover, rendering, sizing, surface};

use layer_shika::calloop::TimeoutAction;
use layer_shika::prelude::*;
use layer_shika::slint_interpreter::Value;
use layer_shika_adapters::AppState;

use std::time::Duration;

const HOVER_POLL_INTERVAL: Duration = Duration::from_millis(33);
const CLOCK_UPDATE_INTERVAL: Duration = Duration::from_secs(1);

pub struct IslandController;

impl IslandController {
    pub fn new() -> Self {
        Self
    }

    pub fn run(self) -> layer_shika::Result<()> {
        let mut shell = surface::create_shell()?;

        // Initial Island theme
        shell.with_component(surface::ISLAND, |instance| {
            let theme = themer::Theme::load();
            themer::apply_theme(instance, &theme);

            println!("Initial theme applied");
        });

        // Trigger initialization
        shell.with_component(surface::TRIGGER, |_instance| {
            println!("Trigger surface initialized");
        });

        let control = shell.control();
        let loop_handle = shell.event_loop_handle();

        // The Island starts collapsed, so it must not block mouse input.
        if let Err(err) = control.surface(surface::ISLAND).set_input_enabled(false) {
            eprintln!("Failed to disable initial Island input: {err}");
        }

        let mut animation = AnimationState::new();
        let mut last_surface_size: Option<(u32, u32)> = None;
        let mut last_date: Option<date::Date> = None;

        // Hover / animation timer
        loop_handle.add_timer(HOVER_POLL_INTERVAL, move |now, app_state: &mut AppState| {
            let hovered_now = hover::is_anything_hovered(app_state);
            let mut dirty = false;

            if hovered_now {
                animation.cancel_close();

                if !animation.is_open {
                    animation.is_open = true;
                    animation.shrink_at = None;

                    // Island is opening, so allow pointer input.
                    if let Err(err) = control.surface(surface::ISLAND).set_input_enabled(true) {
                        eprintln!("Failed to enable Island input: {err}");
                    }

                    surface::set_visible(app_state, true);

                    if resize_if_needed(&control, app_state, &mut last_surface_size) {
                        dirty = true;
                    }
                }

                // Follow Slint content-size changes.
                if resize_if_needed(&control, app_state, &mut last_surface_size) {
                    dirty = true;
                }
            } else if animation.is_open {
                animation.schedule_close(now);
            }

            // Fully hide the Island after the close delay.
            if animation.should_close(now) {
                animation.mark_closed(now);

                surface::set_visible(app_state, false);
                dirty = true;
            }

            // Collapse the Island and make it click-through.
            if animation.should_collapse(now) {
                animation.mark_collapsed();

                // Disable pointer input before/while collapsed.
                if let Err(err) = control.surface(surface::ISLAND).set_input_enabled(false) {
                    eprintln!("Failed to disable collapsed Island input: {err}");
                }

                let (width, height) = sizing::collapsed_size();

                println!("[SIZE] Collapsing Island to {}x{}", width, height);

                if let Err(err) = control.surface(surface::ISLAND).resize(width, height) {
                    eprintln!("Failed to collapse Island: {err}");
                } else {
                    last_surface_size = Some((width, height));
                    dirty = true;
                }
            }

            if dirty {
                rendering::render_dirty(app_state);
            }

            TimeoutAction::ToDuration(HOVER_POLL_INTERVAL)
        })?;

        // Clock / Date timer
        loop_handle.add_timer(
            CLOCK_UPDATE_INTERVAL,
            move |_now, app_state: &mut AppState| {
                update_clock(app_state);

                let current_date = date::Date::now();

                if last_date != Some(current_date) {
                    update_date(app_state, current_date);
                    last_date = Some(current_date);
                }

                TimeoutAction::ToDuration(CLOCK_UPDATE_INTERVAL)
            },
        )?;

        // Theme watcher
        let (_theme_token, theme_sender) =
            loop_handle.add_channel::<(), _>(move |_msg, app_state: &mut AppState| {
                println!("[UI] Reloading theme...");

                let theme = themer::Theme::load();

                for island_surface in app_state.surfaces_by_name_mut(surface::ISLAND) {
                    themer::apply_theme(island_surface.component_instance(), &theme);
                }

                rendering::render_dirty(app_state);

                println!("[UI] Theme applied.");
            })?;

        themer::start_watcher(theme_sender);

        shell.run()?;

        Ok(())
    }
}

fn resize_if_needed(
    control: &ShellControl,
    app_state: &mut AppState,
    last_size: &mut Option<(u32, u32)>,
) -> bool {
    let Some(new_size) = sizing::get_content_size(app_state) else {
        return false;
    };

    if *last_size == Some(new_size) {
        return false;
    }

    println!("[SIZE] Resizing Island to {}x{}", new_size.0, new_size.1);

    if let Err(err) = control
        .surface(surface::ISLAND)
        .resize(new_size.0, new_size.1)
    {
        eprintln!("Failed to resize Island: {err}");
        return false;
    }

    *last_size = Some(new_size);
    true
}

/////////////////////////////////////////////////Updating clock here/////////////////////////////////////////////////////
fn update_clock(app_state: &mut AppState) {
    let time = clock::Clock::now();

    let hour_value = Value::from(time.hour as i32);
    let minute_value = Value::from(time.minute as i32);
    let second_value = Value::from(time.second as i32);

    for island_surface in app_state.surfaces_by_name_mut(surface::ISLAND) {
        let instance = island_surface.component_instance();

        if let Err(err) = instance.set_global_property("ClockState", "hour", hour_value.clone()) {
            eprintln!("Failed to update clock hour: {err}");
        }

        if let Err(err) = instance.set_global_property("ClockState", "minute", minute_value.clone())
        {
            eprintln!("Failed to update clock minute: {err}");
        }

        if let Err(err) =
            instance.set_global_property("ClockState", "seconds", second_value.clone())
        {
            eprintln!("Failed to update clock seconds: {err}");
        }
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////Updating date here////////////////////////////////////////////////////
fn update_date(app_state: &mut AppState, current_date: date::Date) {
    let day_value = Value::from(current_date.day as i32);
    let month_value = Value::from(current_date.month as i32);
    let year_value = Value::from(current_date.year);
    let weekday_value = Value::from(current_date.weekday as i32);

    for island_surface in app_state.surfaces_by_name_mut(surface::ISLAND) {
        let instance = island_surface.component_instance();

        if let Err(err) = instance.set_global_property("DateState", "day", day_value.clone()) {
            eprintln!("Failed to update date day: {err}");
        }

        if let Err(err) = instance.set_global_property("DateState", "month", month_value.clone()) {
            eprintln!("Failed to update date month: {err}");
        }

        if let Err(err) = instance.set_global_property("DateState", "year", year_value.clone()) {
            eprintln!("Failed to update date year: {err}");
        }

        if let Err(err) =
            instance.set_global_property("DateState", "weekday", weekday_value.clone())
        {
            eprintln!("Failed to update date weekday: {err}");
        }
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
