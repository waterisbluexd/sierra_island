use std::rc::Rc;
use std::time::{Duration, Instant};

use smithay_client_toolkit::reexports::calloop::{LoopHandle, RegistrationToken, channel::Sender};
use smithay_client_toolkit::reexports::client::QueueHandle;

use slint::{
    ComponentHandle, ModelRc, PhysicalSize, SharedPixelBuffer, SharedString,
    platform::software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor},
};

use smithay_client_toolkit::{
    compositor::CompositorState, output::OutputState, registry::RegistryState, seat::SeatState,
    shell::wlr_layer::LayerShell, shm::slot::SlotPool,
};

use crate::wayland::{
    output_surface::OutputSurface, rendering::SierraRenderer, surface::COLLAPSED_HEIGHT,
    time::update_time_state,
};

pub const CLOSE_DELAY: Duration = Duration::from_secs(2);
pub const COLLAPSE_DELAY: Duration = Duration::from_millis(220);
pub const HEIGHT_ANIM_DURATION: Duration = Duration::from_millis(200);
pub const MAX_ISLAND_HEIGHT: u32 = 200;

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
    pub loop_handle: LoopHandle<'static, SierraState>,
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
    pub search_results: ModelRc<SharedString>,
    pub search_results_rx: std::sync::mpsc::Receiver<Vec<SharedString>>,
    pub height_target: u32,
    pub height_anim_from: u32,
    pub height_anim_start: Instant,
    pub height_animating: bool,
}

impl SierraState {
    pub fn update_time(&self) {
        update_time_state(&self.island);
    }

    pub fn draw(&mut self) -> bool {
        if !self.configured {
            return false;
        }

        while let Ok(names) = self.search_results_rx.try_recv() {
            self.search_results = names.as_slice().into();
            self.island.set_search_results(self.search_results.clone());
        }

        let (new_width, new_height) = self.content_size();
        let new_width = new_width.max(1);
        let new_height = new_height.clamp(1, MAX_ISLAND_HEIGHT);

        // Recalculate the target whenever the search results change so the animation
        // always interpolates toward the latest content height, in either direction.
        // While the island is hidden/collapsing the target is owned by collapse_island.
        if self.island_visible && new_height != self.height_target {
            self.height_target = new_height;
            self.height_anim_from = self.height;
            self.height_anim_start = Instant::now();
            self.height_animating = true;
        }

        let mut height = self.height;
        if self.height_animating {
            let elapsed = Instant::now().duration_since(self.height_anim_start);
            let t = (elapsed.as_secs_f64() / HEIGHT_ANIM_DURATION.as_secs_f64()).clamp(0.0, 1.0);
            // Ease-out cubic: quick at the start, subtle as it settles.
            let eased = 1.0 - (1.0 - t).powi(3);
            let from = self.height_anim_from as f64;
            let target = self.height_target as f64;
            height = (from + (target - from) * eased).round() as u32;
            if t >= 1.0 {
                self.height_animating = false;
                height = self.height_target;
            }
        }

        if new_width != self.width || height != self.height {
            self.resize_island(new_width, height);
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

        if !self.island_visible {
            self.island.set_visible_requested(true);
            self.island.window().request_redraw();
            self.island_visible = true;
        }
        let target = self.content_size().1.clamp(1, MAX_ISLAND_HEIGHT);
        if target != self.height_target {
            self.height_target = target;
            self.height_anim_from = self.height;
            self.height_anim_start = Instant::now();
            self.height_animating = true;
        }
        self.island_expanded = true;
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

        // Shrink the height back to the compact size through the shared animation
        // path instead of snapping it instantly.
        self.height_target = COLLAPSED_HEIGHT;
        self.height_anim_from = self.height;
        self.height_anim_start = Instant::now();
        self.height_animating = true;

        self.island.set_visible_requested(false);
        self.island.window().request_redraw();
        self.island_visible = false;
        self.island_expanded = false;
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
