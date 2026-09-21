use std::rc::Rc;
use std::cell::RefCell;

use slint::Platform;
use slint::PlatformError;
use slint::platform::software_renderer::SoftwareWindowAdapter;
use slint::platform::software_renderer::SoftwareWindow;

use crate::platform::wayland::WaylandState;

pub struct SierraIslandPlatform {
    state: Rc<RefCell<WaylandState>>,
}

impl SierraIslandPlatform {
    pub fn new() -> Result<Self, PlatformError> {
        let state = Rc::new(RefCell::new(WaylandState::new()?));
        Ok(Self { state })
    }
}

impl Platform for SierraIslandPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn SoftwareWindowAdapter>, PlatformError> {
        Ok(Rc::new(SierraIslandWindowAdapter::new(self.state.clone())))
    }
}

pub struct SierraIslandWindowAdapter {
    state: Rc<RefCell<WaylandState>>,
}

impl SierraIslandWindowAdapter {
    pub fn new(state: Rc<RefCell<WaylandState>>) -> Self {
        Self { state }
    }
}

impl SoftwareWindowAdapter for SierraIslandWindowAdapter {
    fn create_window(&self) -> Result<SoftwareWindow, PlatformError> {
        let window = SierraIslandSoftwareWindow::new(self.state.clone());
        Ok(window)
    }
}

pub struct SierraIslandSoftwareWindow {
    width: u32,
    height: u32,
    state: Rc<RefCell<WaylandState>>,
}

impl SierraIslandSoftwareWindow {
    pub fn new(state: Rc<RefCell<WaylandState>>) -> Self {
        let mut window = Self {
            width: 0,
            height: 0,
            state,
        };
        window.create_surfaces();
        window
    }
    
    fn create_surfaces(&mut self) {
        let mut state = self.state.borrow_mut();
        
        // Create Trigger surface
        let trigger_surface = state.compositor.create_surface();
        let trigger_layer = state.shell.create_layer_surface(
            "Trigger",
            Anchor::Top,
            Layer::Background,
            ExclusiveZone(0),
            Margin { top: 0, left: 0, right: 0, bottom: 0 },
        );
        trigger_surface.attach_to_layer_surface(trigger_layer);
        
        state.surfaces_by_name.insert("Trigger".to_string(), SurfaceState {
            handle: state.next_surface_handle,
            name: "Trigger".to_string(),
            component_instance: ComponentInstance::new(),
            width: 260,
            height: 4,
            input_enabled: false,
            is_visible: false,
            wl_surface: trigger_surface,
            layer_surface: Some(trigger_layer),
        });
        state.next_surface_handle += 1;
        
        // Create Island surface
        let island_surface = state.compositor.create_surface();
        let island_layer = state.shell.create_layer_surface(
            "Island",
            Anchor::Top,
            Layer::Background,
            ExclusiveZone(0),
            Margin { top: 0, left: 0, right: 0, bottom: 0 },
        );
        island_surface.attach_to_layer_surface(island_layer);
        
        state.surfaces_by_name.insert("Island".to_string(), SurfaceState {
            handle: state.next_surface_handle,
            name: "Island".to_string(),
            component_instance: ComponentInstance::new(),
            width: 250,
            height: 1,
            input_enabled: false,
            is_visible: false,
            wl_surface: island_surface,
            layer_surface: Some(island_layer),
        });
        state.next_surface_handle += 1;
    }
}

impl SoftwareWindow for SierraIslandSoftwareWindow {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn scale_factor(&self) -> f32 {
        1.0
    }

    fn render<F: FnOnce(&mut SoftwareRenderer)>(&mut self, f: F) {
        // Render callback
    }

    fn request_redraw(&mut self) {
        // Request redraw
    }
}
