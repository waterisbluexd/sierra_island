use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, PhysicalSize, SharedPixelBuffer, platform::software_renderer::{MinimalSoftwareWindow, PremultipliedRgbaColor}};

use smithay_client_toolkit::{
    compositor::CompositorHandler,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        WaylandSurface,
        wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    },
    shm::{
        ShmHandler,
        slot::{Buffer, SlotPool},
    },
};

use crate::wayland::{
    time::update_time_state,
    rendering::SierraRenderer,
    surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH, HEIGHT, TRIGGER_HEIGHT, TRIGGER_WIDTH, WIDTH},
};

pub const CLOSE_DELAY: Duration = Duration::from_secs(2);
pub const COLLAPSE_DELAY: Duration = Duration::from_millis(220);

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

    pub hide_at: Option<Instant>,

    pub collapse_at: Option<Instant>,

    pub exit: bool,
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
        self.hide_at = None;
        self.collapse_at = None;

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

    pub fn start_close(&mut self, now: Instant) {
        if !self.island_expanded || self.hide_at.is_some() {
            return;
        }

        self.hide_at = Some(now + CLOSE_DELAY);
    }

    pub fn hide_island(&mut self, now: Instant) {
        if !self.island_expanded {
            return;
        }

        self.hide_at = None;

        self.island.set_visible_requested(false);
        self.island.window().request_redraw();

        self.island_visible = false;

        self.collapse_at = Some(now + COLLAPSE_DELAY);
    }

    pub fn collapse_island(&mut self) {
        if !self.island_expanded {
            return;
        }

        self.collapse_at = None;

        self.resize_island(COLLAPSED_WIDTH, COLLAPSED_HEIGHT);

        self.island_expanded = false;
        self.island_visible = false;
    }

    pub fn update_hover(&mut self, now: Instant) {
        let hovered = self.trigger_hovered || self.island_hovered;

        if hovered {
            self.hide_at = None;
            self.collapse_at = None;

            self.show_island();

            return;
        }

        if self.island_expanded {
            self.start_close(now);
        }

        if let Some(hide_at) = self.hide_at {
            if now >= hide_at {
                self.hide_island(now);
            }
        }

        if let Some(collapse_at) = self.collapse_at {
            if now >= collapse_at && !hovered {
                self.collapse_island();
            }
        }
    }
}

impl CompositorHandler for SierraState {
    fn scale_factor_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _new_transform: smithay_client_toolkit::reexports::client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for SierraState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for SierraState {
    fn closed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        self.exit = true;
        self.loop_signal.stop();
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let is_trigger = layer.wl_surface() == self.trigger_layer.wl_surface();

        if is_trigger {
            self.trigger_configured = true;
            self.attach_trigger_buffer();
            return;
        }

        if configure.new_size.0 != 0 {
            self.width = configure.new_size.0;
        }

        if configure.new_size.1 != 0 {
            self.height = configure.new_size.1;
        }

        self.configured = true;

        self.slint_window
            .set_size(PhysicalSize::new(self.width, self.height));

        self.draw();
    }
}

impl SeatHandler for SierraState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");

            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointer.take();
        }
    }

    fn remove_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl ShmHandler for SierraState {
    fn shm_state(&mut self) -> &mut smithay_client_toolkit::shm::Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for SierraState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    smithay_client_toolkit::registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_registry!(SierraState);
smithay_client_toolkit::delegate_dispatch2!(SierraState);
