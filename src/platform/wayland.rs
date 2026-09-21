use std::rc::Rc;
use std::cell::RefCell;

use smithay_client_toolkit::calloop::Calloop;
use smithay_client_toolkit::wayland::WlCompositor;
use smithay_client_toolkit::wayland::WlShm;
use smithay_client_toolkit::wayland::WlOutput;
use smithay_client_toolkit::wayland::WlSeat;
use smithay_client_toolkit::wayland::WlPointer;
use smithay_client_toolkit::wayland::WlKeyboard;
use smithay_client_toolkit::layer_shell::LayerShellV1;
use smithay_client_toolkit::layer_shell::Anchor;
use smithay_client_toolkit::layer_shell::Layer;

use slint::Platform;
use slint::PlatformError;
use slint::platform::software_renderer::SoftwareWindowAdapter;
use slint::platform::software_renderer::SoftwareWindow;

use crate::components::{ContainerRegistry, Container, WidgetSlot, WidgetType};
use crate::themer::{Theme, apply_theme};
use crate::widgets::{clock::Clock, date::Date};

pub type OutputHandle = u32;
pub type SurfaceHandle = u32;

pub struct AppState {
    pub surfaces_by_name: HashMap<String, SurfaceState>,
    pub surfaces_by_handle: HashMap<SurfaceHandle, SurfaceState>,
    pub outputs: HashMap<OutputHandle, OutputState>,
    pub active_output: Option<OutputHandle>,
    pub active_surface_handle: Option<SurfaceHandle>,
    pub shared_pointer_serial: Rc<RefCell<SharedPointerSerial>>,
    pub output_registry: OutputRegistry,
    pub shell_surface_names: Vec<String>,
}

pub struct SurfaceState {
    pub handle: SurfaceHandle,
    pub name: String,
    pub component_instance: ComponentInstance,
    pub width: u32,
    pub height: u32,
    pub input_enabled: bool,
    pub is_visible: bool,
    pub wl_surface: WlSurface,
    pub layer_surface: Option<LayerSurfaceV1>,
}

pub struct OutputState {
    pub output: WlOutput,
    pub handle: OutputHandle,
    pub surfaces: Vec<SurfaceState>,
}

pub struct OutputRegistry {
    pub outputs: Vec<OutputState>,
    pub next_output_id: OutputHandle,
}

pub struct WaylandState {
    pub event_loop: Calloop,
    pub compositor: WlCompositor,
    pub shell: LayerShellV1,
    pub shm: WlShm,
    pub seat: WlSeat,
    pub pointer: WlPointer,
    pub keyboard: WlKeyboard,
    pub outputs: HashMap<OutputHandle, OutputState>,
    pub surfaces_by_name: HashMap<String, SurfaceState>,
    pub next_surface_handle: SurfaceHandle,
}

impl WaylandState {
    fn new() -> Result<Self, PlatformError> {
        let event_loop = Calloop::new().map_err(|e| PlatformError::Other(e.to_string()))?;
        let (compositor, shell, shm, seat, pointer, keyboard) = (
            WlCompositor::new(),
            LayerShellV1::new(),
            WlShm::new(),
            WlSeat::new(),
            WlPointer::new(),
            WlKeyboard::new(),
        );
        Ok(Self {
            event_loop,
            compositor,
            shell,
            shm,
            seat,
            pointer,
            keyboard,
            outputs: HashMap::new(),
            surfaces_by_name: HashMap::new(),
            next_surface_handle: 1,
        })
    }
}

// Mock types for compilation
pub struct WlSurface;
pub struct WlBuffer;
pub struct LayerSurfaceV1;
pub struct ComponentInstance;

impl ComponentInstance {
    fn new() -> Self { Self }
    fn set_property<T: Into<Value>>(&self, _prop: &str, _value: T) -> Result<(), PlatformError> { Ok(()) }
    fn get_property(&self, _prop: &str) -> Result<Value, PlatformError> { Ok(Value::Boolean(false)) }
    fn set_global_property<T: Into<Value>>(&self, _category: &str, _prop: &str, _value: T) -> Result<(), PlatformError> { Ok(()) }
}

pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Color(slint::Color),
    Brush(slint::Brush),
}

use std::collections::HashMap;

use slint::platform::software_renderer::SoftwareRenderer;

pub type SharedPointerSerial = u32;
