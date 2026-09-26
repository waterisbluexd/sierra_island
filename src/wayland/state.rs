use std::rc::Rc;
use std::time::{Duration, Instant};

use smithay_client_toolkit::reexports::calloop::{
    RegistrationToken,
    channel::Sender,
    timer::{TimeoutAction, Timer},
};

use slint::{ComponentHandle, PhysicalSize, SharedPixelBuffer, platform::software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor}};

use smithay_client_toolkit::{
    output::OutputState,
    registry::RegistryState,
    seat::SeatState,
    shell::{
        WaylandSurface,
        wlr_layer::LayerSurface,
    },
    shm::slot::{Buffer, SlotPool},
};

use crate::wayland::{
    time::update_time_state,
    rendering::SierraRenderer,
    surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH, TRIGGER_HEIGHT, TRIGGER_WIDTH},
};

pub const CLOSE_DELAY: Duration = Duration::from_secs(2);
pub const COLLAPSE_DELAY: Duration = Duration::from_millis(220);
pub const DEFAULT_REFRESH_MHZ: i32 = 60000;

#[derive(Debug, Clone, Copy)]
pub enum EventCommand {
    PointerEnter,
    PointerLeave,
    EnsureTicker,
    ScheduleCollapseTimer,
}

pub struct SierraState {
    pub registry_state: RegistryState,

    pub seat_state: SeatState,

    pub output_state: OutputState,

    pub shm: smithay_client_toolkit::shm::Shm,

    pub layer: LayerSurface,

    pub trigger_layer: LayerSurface,

    pub pool: SlotPool,

    pub buffer: Option<Buffer>,

    pub trigger_buffer: Option<Buffer>,

    pub slint_buffer: Option<SharedPixelBuffer<PremultipliedRgbaColor>>,

    pub slint_window: Rc<MinimalSoftwareWindow>,

    pub island: crate::Island,

    pub pointer: Option<smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer>,

    pub loop_signal: smithay_client_toolkit::reexports::calloop::LoopSignal,

    pub width: u32,

    pub height: u32,

    pub configured: bool,

    pub trigger_configured: bool,

    pub trigger_hovered: bool,

    pub island_hovered: bool,

    pub island_expanded: bool,

    pub island_visible: bool,

    pub exit: bool,

    pub output_id: Option<u32>,

    pub refresh_rate_mhz: i32,

    pub command_sender: Option<Sender<EventCommand>>,

    pub close_timer_pending: bool,

    pub collapse_timer_pending: bool,

    pub animation_ticker_running: bool,

    pub close_timer_token: Option<RegistrationToken>,

    pub collapse_timer_token: Option<RegistrationToken>,

    pub animation_ticker_token: Option<RegistrationToken>,
}

impl SierraState {
    pub fn update_time(&self) {
        update_time_state(&self.island);
    }

    pub fn draw(&mut self) -> bool {
        <Self as SierraRenderer>::draw(self)
    }

    pub fn content_size(&self) -> (u32, u32) {
        let width = self.island.get_content_width().ceil() as u32;
        let height = self.island.get_content_height().ceil() as u32;

        (width.max(1), height.max(1))
    }

    pub fn resize_island(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        self.slint_window.set_size(PhysicalSize::new(width, height));

        self.buffer = None;

        self.slint_buffer = None;

        self.layer.set_size(width, height);
        self.layer.commit();

        self.island.window().request_redraw();
    }

    pub fn attach_trigger_buffer(&mut self) {
        if self.trigger_buffer.is_none() {
            let (buffer, canvas) = self
                .pool
                .create_buffer(
                    TRIGGER_WIDTH as i32,
                    TRIGGER_HEIGHT as i32,
                    TRIGGER_WIDTH as i32 * 4,
                    smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
                )
                .expect("Failed to create trigger wl_shm buffer");

            for byte in canvas.iter_mut() {
                *byte = 0;
            }

            self.trigger_buffer = Some(buffer);
        }

        let buffer = self
            .trigger_buffer
            .as_ref()
            .expect("Trigger buffer missing");

        buffer
            .attach_to(self.trigger_layer.wl_surface())
            .expect("Failed to attach trigger buffer");

        self.trigger_layer.wl_surface().damage_buffer(
            0,
            0,
            TRIGGER_WIDTH as i32,
            TRIGGER_HEIGHT as i32,
        );

        self.trigger_layer.commit();
    }

    pub fn show_island(&mut self) {
        self.close_timer_pending = false;
        self.collapse_timer_pending = false;

        if !self.island_expanded {
            let (width, height) = self.content_size();

            self.resize_island(width, height);

            self.island_expanded = true;
        }

        if !self.island_visible {
            self.island.set_visible_requested(true);
            self.island.window().request_redraw();

            self.island_visible = true;
        }
    }

    pub fn hide_island(&mut self) {
        if !self.island_expanded {
            return;
        }

        self.island.set_visible_requested(false);
        self.island.window().request_redraw();

        self.island_visible = false;
    }

    pub fn collapse_island(&mut self) {
        if !self.island_expanded {
            return;
        }

        self.resize_island(COLLAPSED_WIDTH, COLLAPSED_HEIGHT);

        self.island_expanded = false;
        self.island_visible = false;
    }

    pub fn update_hover(&mut self, now: Instant) {
        let hovered = self.trigger_hovered || self.island_hovered;

        if hovered {
            self.show_island();
        } else if self.island_expanded {
            self.start_close(now);
        }
    }

    pub fn start_close(&mut self, _now: Instant) {
        if !self.island_expanded || self.close_timer_pending {
            return;
        }

        self.close_timer_pending = true;
    }

    pub fn refresh_period(&self) -> Duration {
        let mhz = self.refresh_rate_mhz.max(1000);
        let nanos = 1_000_000_000_000 / mhz as u64;
        Duration::from_nanos(nanos)
    }

    pub fn has_active_animations(&self) -> bool {
        self.island.window().has_active_animations()
    }

    pub fn duration_until_next_timer_update(&self) -> Option<Duration> {
        slint::platform::duration_until_next_timer_update()
    }

    pub fn update_refresh_rate(&mut self, output_id: u32, info: &smithay_client_toolkit::output::OutputInfo) {
        self.output_id = Some(output_id);
        self.refresh_rate_mhz = info
            .modes
            .iter()
            .find(|m| m.current)
            .map(|m| m.refresh_rate)
            .filter(|&mhz| mhz > 0)
            .unwrap_or(DEFAULT_REFRESH_MHZ);
    }

    pub fn clear_output(&mut self) {
        self.output_id = None;
    }

    pub fn ensure_ticker_running<'l>(
        &mut self,
        loop_handle: &smithay_client_toolkit::reexports::calloop::LoopHandle<'l, Self>,
    ) {
        if self.animation_ticker_running {
            return;
        }

        if !self.has_active_animations() && self.duration_until_next_timer_update().is_none() {
            return;
        }

        self.animation_ticker_running = true;

        let token = loop_handle
            .insert_source(
                Timer::from_duration(self.refresh_period()),
                |_instant, _, state| {
                    slint::platform::update_timers_and_animations();
                    let _ = state.draw();

                    let has_animations = state.has_active_animations();
                    let next_timer = state.duration_until_next_timer_update();

                    if has_animations {
                        TimeoutAction::ToDuration(state.refresh_period())
                    } else if let Some(duration) = next_timer {
                        TimeoutAction::ToDuration(duration)
                    } else {
                        state.animation_ticker_running = false;
                        state.animation_ticker_token = None;
                        TimeoutAction::Drop
                    }
                },
            )
            .ok();

        self.animation_ticker_token = token;
    }
}
