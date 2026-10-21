use smithay_client_toolkit::{
    compositor::CompositorHandler,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    seat::{Capability, SeatHandler, SeatState},
    shell::{WaylandSurface, wlr_layer::LayerShellHandler},
    shm::ShmHandler,
};

use crate::wayland::state::SierraState;

use slint::ComponentHandle;

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
        output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        self.add_output(output);
        self.recompute_refresh_rate();
    }

    fn update_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        self.recompute_refresh_rate();
    }

    fn output_destroyed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        self.remove_output(&output);
        self.recompute_refresh_rate();
    }
}

impl LayerShellHandler for SierraState {
    fn closed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
        self.remove_layer(layer);
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let Some(idx) = self.surfaces.iter().position(|s| {
            s.layer.wl_surface() == layer.wl_surface()
                || s.trigger_layer.wl_surface() == layer.wl_surface()
        }) else {
            return;
        };

        if self.surfaces[idx].trigger_layer.wl_surface() == layer.wl_surface() {
            self.surfaces[idx].trigger_configured = true;
            self.attach_trigger_buffer(idx);
            return;
        }

        self.surfaces[idx].configured = true;
        self.configured = true;

        // Outputs that aren't showing the island just get a transparent buffer.
        if !self.surfaces[idx].active {
            self.attach_blank(idx);
            return;
        }

        self.slint_window
            .set_size(slint::PhysicalSize::new(self.width, self.height));

        // A newly configured output has no buffer yet; force a fresh render
        // so it gets one.
        self.island.window().request_redraw();

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
