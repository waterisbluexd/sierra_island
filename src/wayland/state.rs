use std::rc::Rc;
use std::time::{Duration, Instant};

use smithay_client_toolkit::reexports::calloop::{RegistrationToken, channel::Sender};
use smithay_client_toolkit::reexports::client::QueueHandle;

use slint::{
    ComponentHandle, PhysicalSize, SharedPixelBuffer,
    platform::software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor},
};

use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell, shm::slot::SlotPool,
};

use crate::wayland::{
    output_surface::OutputSurface,
    rendering::SierraRenderer,
    surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH},
    time::update_time_state,
};

pub const CLOSE_DELAY: Duration = Duration::from_secs(2);
pub const COLLAPSE_DELAY: Duration = Duration::from_millis(220);

#[derive(Debug, Clone, Copy)]
pub enum EventCommand {
    PointerEnter,
    PointerLeave,
    ScheduleCollapseTimer,
}

pub struct SierraState {
    pub registry_state: RegistryState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub qh: QueueHandle<SierraState>,
    pub surfaces: Vec<OutputSurface>,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: smithay_client_toolkit::shm::Shm,
    pub pool: SlotPool,
    pub slint_buffer: Option<SharedPixelBuffer<PremultipliedRgbaColor>>,
    pub slint_window: Rc<MinimalSoftwareWindow>,
    pub island: crate::Island,
    pub pointer: Option<smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer>,
    pub keyboard:
        Option<smithay_client_toolkit::reexports::client::protocol::wl_keyboard::WlKeyboard>,
    pub island_keyboard_entered: bool,
    pub width: u32,
    pub height: u32,
    pub configured: bool,
    pub trigger_hovered: bool,
    pub island_hovered: bool,
    pub island_expanded: bool,
    pub island_visible: bool,
    pub refresh_rate_mhz: i32,
    pub command_sender: Option<Sender<EventCommand>>,
    pub close_timer_pending: bool,
    pub collapse_timer_pending: bool,
    pub close_timer_token: Option<RegistrationToken>,
    pub collapse_timer_token: Option<RegistrationToken>,
}

impl SierraState {
    pub fn update_time(&self) {
        update_time_state(&self.island);
    }

    pub fn draw(&mut self) -> bool {
        if !self.configured {
            return false;
        }

        let (new_width, new_height) = self.content_size();
        if new_width != self.width || new_height != self.height {
            self.resize_island(new_width, new_height);
        }

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

        self.slint_buffer = None;

        for surface in self.surfaces.iter_mut().filter(|s| s.active) {
            surface.buffer = None;
        }

        self.sync_surface_sizes();

        self.island.window().request_redraw();
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

    pub fn recompute_refresh_rate(&mut self) {
        self.refresh_rate_mhz = self
            .output_state
            .outputs()
            .filter_map(|output| self.output_state.info(&output))
            .filter_map(|info| {
                info.modes
                    .iter()
                    .find(|m| m.current)
                    .map(|m| m.refresh_rate)
            })
            .filter(|&mhz| mhz > 0)
            .max()
            .unwrap_or(60000);
    }
}
