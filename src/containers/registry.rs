use super::{container::Container, widget_slot::WidgetSlot};
use crate::containers::ClickAction::Show;
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

    pub fn get(&self, id: u32) -> Option<&Container> {
        self.containers.iter().find(|container| container.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Container> {
        self.containers
            .iter_mut()
            .find(|container| container.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Container> {
        self.containers.iter()
    }

    pub fn len(&self) -> usize {
        self.containers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
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
