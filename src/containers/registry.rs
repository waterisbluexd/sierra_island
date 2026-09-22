use super::{container::Container, widget_slot::WidgetSlot};
use crate::widgets::clock::{ClockConfig, TimeFormat};

#[derive(Debug, Default)]
pub struct ContainerRegistry {
    containers: Vec<Container>,
}

impl ContainerRegistry {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    pub fn add(&mut self, container: Container) -> bool {
        if self
            .containers
            .iter()
            .any(|existing| existing.id == container.id)
        {
            return false;
        }

        self.containers.push(container);
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = &Container> {
        self.containers.iter()
    }

    pub fn default_layout() -> Self {
        let mut registry = Self::new();

        registry.add(Container::new(1, "Main Container").slot(
            WidgetSlot::new(1).clock_with_config(ClockConfig {
                format: TimeFormat::TwelveHour,
                show_seconds: false,
                show_am_pm: true,
            }),
        ));

        registry.add(Container::new(2, "System Container").slot(WidgetSlot::new(3).date()));

        registry
    }
}
