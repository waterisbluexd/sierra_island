use crate::widgets::{clock::ClockConfig, date::DateConfig, input::InputConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Clock(ClockConfig),
    Date(DateConfig),
    Input(InputConfig),
}

#[derive(Debug, Clone)]
pub struct WidgetSlot {
    pub id: u32,
    pub widgets: Vec<WidgetType>,
}

impl WidgetSlot {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            widgets: Vec::new(),
        }
    }

    pub fn widget(mut self, widget: WidgetType) -> Self {
        self.widgets.push(widget);
        self
    }

    pub fn clock_with_config(self, config: ClockConfig) -> Self {
        self.widget(WidgetType::Clock(config))
    }

    pub fn date(self) -> Self {
        self.date_with_config(DateConfig::default())
    }

    pub fn date_with_config(self, config: DateConfig) -> Self {
        self.widget(WidgetType::Date(config))
    }

    pub fn input(self) -> Self {
        self.input_with_config(InputConfig::new())
    }

    pub fn input_with_config(self, config: InputConfig) -> Self {
        self.widget(WidgetType::Input(config))
    }
}
