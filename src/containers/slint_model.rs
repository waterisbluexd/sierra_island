use crate::{
    ContainerConfig, WidgetConfig, WidgetSlotConfig,
    containers::{Container, ContainerRegistry, WidgetType},
};

use slint::{ModelRc, SharedString};

pub fn container_registry_to_model(registry: &ContainerRegistry) -> ModelRc<ContainerConfig> {
    let containers: Vec<ContainerConfig> = registry.iter().map(container_to_config).collect();

    containers.as_slice().into()
}

fn container_to_config(container: &Container) -> ContainerConfig {
    let slots: Vec<WidgetSlotConfig> = container.slots.iter().map(slot_to_config).collect();

    ContainerConfig {
        id: container.id as i32,
        name: SharedString::from(container.name.clone()),
        slots: slots.as_slice().into(),
    }
}

fn slot_to_config(slot: &crate::containers::WidgetSlot) -> WidgetSlotConfig {
    let widgets: Vec<WidgetConfig> = slot.widgets.iter().map(widget_to_config).collect();

    WidgetSlotConfig {
        id: slot.id as i32,
        widgets: widgets.as_slice().into(),
    }
}

fn widget_to_config(widget: &WidgetType) -> WidgetConfig {
    match widget {
        WidgetType::Clock(config) => WidgetConfig {
            widget_type: 0,
            show_seconds: config.show_seconds,
            is_24_hour: matches!(
                config.format,
                crate::widgets::clock::TimeFormat::TwentyFourHour
            ),
            show_am_pm: config.show_am_pm,
            date_format: SharedString::new(),
        },

        WidgetType::Date(config) => WidgetConfig {
            widget_type: 1,
            show_seconds: false,
            is_24_hour: false,
            show_am_pm: false,
            date_format: SharedString::from(config.format.as_str()),
        },
    }
}
