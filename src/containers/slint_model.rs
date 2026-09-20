use crate::containers::{Container, ContainerRegistry, WidgetType};

use layer_shika::slint_interpreter::{SharedString, Struct, Value};

use slint::ModelRc;

pub fn container_registry_to_value(registry: &ContainerRegistry) -> Value {
    let containers: Vec<Value> = registry.iter().map(container_to_value).collect();

    Value::from(ModelRc::from(containers.as_slice()))
}

fn container_to_value(container: &Container) -> Value {
    let slots: Vec<Value> = container.slots.iter().map(slot_to_value).collect();

    Struct::from_iter([
        ("id".to_owned(), Value::from(container.id as i32)),
        (
            "name".to_owned(),
            Value::from(SharedString::from(container.name.clone())),
        ),
        (
            "slots".to_owned(),
            Value::from(ModelRc::from(slots.as_slice())),
        ),
    ])
    .into()
}

fn slot_to_value(slot: &crate::containers::WidgetSlot) -> Value {
    let widgets: Vec<Value> = slot.widgets.iter().map(widget_to_value).collect();

    Struct::from_iter([
        ("id".to_owned(), Value::from(slot.id as i32)),
        (
            "widgets".to_owned(),
            Value::from(ModelRc::from(widgets.as_slice())),
        ),
    ])
    .into()
}

fn widget_to_value(widget: &WidgetType) -> Value {
    match widget {
        WidgetType::Clock(config) => {
            let is_24_hour = matches!(
                config.format,
                crate::widgets::clock::TimeFormat::TwentyFourHour
            );

            Struct::from_iter([
                ("widget-type".to_owned(), Value::from(0i32)),
                ("show-seconds".to_owned(), Value::from(config.show_seconds)),
                ("is-24-hour".to_owned(), Value::from(is_24_hour)),
                ("show-am-pm".to_owned(), Value::from(config.show_am_pm)),
                ("date-format".to_owned(), Value::from(SharedString::new())),
            ])
            .into()
        }

        WidgetType::Date(config) => Struct::from_iter([
            ("widget-type".to_owned(), Value::from(1i32)),
            ("show-seconds".to_owned(), Value::from(false)),
            ("is-24-hour".to_owned(), Value::from(false)),
            ("show-am-pm".to_owned(), Value::from(false)),
            (
                "date-format".to_owned(),
                Value::from(SharedString::from(config.format.as_str())),
            ),
        ])
        .into(),
    }
}
