use crate::platform::wayland::WaylandState;
use crate::platform::slint_platform::SierraIslandPlatform;

pub struct SierraIslandLauncher {
    platform: SierraIslandPlatform,
}

impl SierraIslandLauncher {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let platform = SierraIslandPlatform::new()?;
        Ok(Self { platform })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Slint with our custom platform
        let mut slint_app = slint::Application::new()?;
        
        // Set our platform
        slint_app.set_platform(self.platform.clone());
        
        // Create the UI component
        let component = slint::Component::new("ui/island.slint")?;
        
        // Run the application
        slint_app.run()?;
        Ok(())
    }
}
