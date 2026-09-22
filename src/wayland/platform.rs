use std::rc::Rc;

use slint::platform::{
    Platform, PlatformError,
    software_renderer::MinimalSoftwareWindow,
};

pub struct SierraPlatform {
    pub window: Rc<MinimalSoftwareWindow>,
}

impl Platform for SierraPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn slint::platform::WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}
