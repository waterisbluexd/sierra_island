use super::widget_slot::WidgetSlot;

#[derive(Debug, Clone)]
pub struct Container {
    pub id: u32,
    pub name: String,
    pub slots: Vec<WidgetSlot>,
}

impl Container {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            slots: Vec::new(),
        }
    }

    pub fn slot(mut self, slot: WidgetSlot) -> Self {
        self.slots.push(slot);
        self
    }
}
