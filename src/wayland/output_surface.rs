use smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput;

use smithay_client_toolkit::{
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerSurface},
    },
    shm::slot::Buffer,
};

use slint::ComponentHandle;

use crate::wayland::{
    state::SierraState,
    surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH, TRIGGER_HEIGHT, TRIGGER_WIDTH},
};

pub struct OutputSurface {
    pub output: WlOutput,
    pub layer: LayerSurface,
    pub trigger_layer: LayerSurface,
    pub buffer: Option<Buffer>,
    pub trigger_buffer: Option<Buffer>,
    pub configured: bool,
    pub trigger_configured: bool,
    pub trigger_hovered: bool,
    pub island_hovered: bool,
    pub active: bool,
    pub size: (u32, u32),
}

impl SierraState {
    pub fn add_output(&mut self, output: WlOutput) {
        if self.surfaces.iter().any(|s| s.output == output) {
            return;
        }

        let active = self.surfaces.is_empty();

        let size = if active {
            (self.width, self.height)
        } else {
            (COLLAPSED_WIDTH, COLLAPSED_HEIGHT)
        };

        let surface = self.compositor_state.create_surface(&self.qh);

        let layer = self.layer_shell.create_layer_surface(
            &self.qh,
            surface,
            Layer::Top,
            Some("sierra_island"),
            Some(&output),
        );

        layer.set_anchor(Anchor::TOP);
        layer.set_size(size.0, size.1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_exclusive_zone(0);
        layer.set_margin(2, 0, 0, 0);
        layer.commit();

        let trigger_surface = self.compositor_state.create_surface(&self.qh);

        let trigger_layer = self.layer_shell.create_layer_surface(
            &self.qh,
            trigger_surface,
            Layer::Top,
            Some("sierra_island_trigger"),
            Some(&output),
        );

        trigger_layer.set_anchor(Anchor::TOP);
        trigger_layer.set_size(TRIGGER_WIDTH, TRIGGER_HEIGHT);
        trigger_layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        trigger_layer.set_exclusive_zone(0);
        trigger_layer.set_margin(0, 0, 0, 0);
        trigger_layer.commit();

        self.surfaces.push(OutputSurface {
            output,
            layer,
            trigger_layer,
            buffer: None,
            trigger_buffer: None,
            configured: false,
            trigger_configured: false,
            trigger_hovered: false,
            island_hovered: false,
            active,
            size,
        });
    }

    pub fn remove_output(&mut self, output: &WlOutput) {
        self.surfaces.retain(|s| &s.output != output);
        self.refresh_aggregates();
    }

    pub fn remove_layer(&mut self, layer: &LayerSurface) {
        self.surfaces.retain(|s| {
            s.layer.wl_surface() != layer.wl_surface()
                && s.trigger_layer.wl_surface() != layer.wl_surface()
        });
        self.refresh_aggregates();
    }

    /// Recompute the global flags from the per-output state.
    pub fn refresh_aggregates(&mut self) {
        self.configured = self.surfaces.iter().any(|s| s.configured);
        self.trigger_hovered = self.surfaces.iter().any(|s| s.trigger_hovered);
        self.island_hovered = self.surfaces.iter().any(|s| s.island_hovered);

        if !self.surfaces.is_empty() && !self.surfaces.iter().any(|s| s.active) {
            self.surfaces[0].active = true;
            self.sync_surface_sizes();
            self.island.window().request_redraw();
        }
    }

    /// Show the island only on the output the pointer entered.
    pub fn set_active(&mut self, idx: usize) {
        if idx >= self.surfaces.len() || self.surfaces[idx].active {
            return;
        }

        for (i, surface) in self.surfaces.iter_mut().enumerate() {
            surface.active = i == idx;
        }

        self.sync_surface_sizes();
        self.island.window().request_redraw();
    }

    /// Make every output's surface the size it should be: the active one gets
    /// the full island, all others stay collapsed and empty.
    pub fn sync_surface_sizes(&mut self) {
        let (width, height) = (self.width, self.height);

        for surface in self.surfaces.iter_mut() {
            let want = if surface.active {
                (width, height)
            } else {
                (COLLAPSED_WIDTH, COLLAPSED_HEIGHT)
            };

            if surface.size != want {
                surface.size = want;
                surface.buffer = None;
                surface.layer.set_size(want.0, want.1);
                surface.layer.commit();
            }
        }
    }

    /// Give an output a transparent buffer so nothing is visible on it.
    pub fn attach_blank(&mut self, idx: usize) {
        let (width, height) = self.surfaces[idx].size;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                width as i32 * 4,
                smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
            )
            .expect("Failed to create blank wl_shm buffer");

        canvas.fill(0);

        let surface = &mut self.surfaces[idx];

        buffer
            .attach_to(surface.layer.wl_surface())
            .expect("Failed to attach blank buffer");

        surface
            .layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        surface.layer.commit();

        surface.buffer = Some(buffer);
    }

    pub fn attach_trigger_buffer(&mut self, idx: usize) {
        if self.surfaces[idx].trigger_buffer.is_none() {
            let (buffer, canvas) = self
                .pool
                .create_buffer(
                    TRIGGER_WIDTH as i32,
                    TRIGGER_HEIGHT as i32,
                    TRIGGER_WIDTH as i32 * 4,
                    smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
                )
                .expect("Failed to create trigger wl_shm buffer");

            canvas.fill(0);

            self.surfaces[idx].trigger_buffer = Some(buffer);
        }

        let surface = &self.surfaces[idx];

        let buffer = surface
            .trigger_buffer
            .as_ref()
            .expect("Trigger buffer missing");

        buffer
            .attach_to(surface.trigger_layer.wl_surface())
            .expect("Failed to attach trigger buffer");

        surface.trigger_layer.wl_surface().damage_buffer(
            0,
            0,
            TRIGGER_WIDTH as i32,
            TRIGGER_HEIGHT as i32,
        );

        surface.trigger_layer.commit();
    }
}
