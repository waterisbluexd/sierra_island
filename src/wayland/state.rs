use std::rc::Rc;


use smithay_client_toolkit::{
    compositor::CompositorHandler,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    seat::{Capability, SeatHandler, SeatState},
    shell::wlr_layer::{LayerShellHandler, LayerSurface, LayerSurfaceConfigure},
    shm::ShmHandler,
};

use slint::PhysicalSize;
use slint::platform::software_renderer::MinimalSoftwareWindow;

use crate::wayland::clock::update_time_state;
use crate::wayland::rendering::SierraRenderer;

pub struct SierraState {
    pub registry_state: RegistryState,

    pub seat_state: SeatState,

    pub output_state: OutputState,

    pub shm: smithay_client_toolkit::shm::Shm,

    pub layer: LayerSurface,

    pub pool: smithay_client_toolkit::shm::slot::SlotPool,

    pub buffer: Option<smithay_client_toolkit::shm::slot::Buffer>,

    pub slint_window: Rc<MinimalSoftwareWindow>,

    pub island: crate::Island,

    pub pointer: Option<smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer>,

    pub loop_signal: smithay_client_toolkit::reexports::calloop::LoopSignal,

    pub width: u32,

    pub height: u32,

    pub configured: bool,

    pub exit: bool,
}

impl SierraState {
    pub fn update_time(&self) {
        update_time_state(&self.island);
    }

    pub fn draw(&mut self) -> bool {
        <Self as SierraRenderer>::draw(self)
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
    fn closed(&mut self, _conn: &smithay_client_toolkit::reexports::client::Connection, _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
        self.loop_signal.stop();
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
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

    fn new_seat(&mut self, _conn: &smithay_client_toolkit::reexports::client::Connection, _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>, _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat) {}

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

    fn remove_seat(&mut self, _conn: &smithay_client_toolkit::reexports::client::Connection, _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>, _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat) {}
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
