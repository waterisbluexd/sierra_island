#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub direction: LayoutDirection,
    pub alignment: Alignment,
    pub spacing: u32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            width: 150,
            height: 36,
            direction: LayoutDirection::Horizontal,
            alignment: Alignment::Center,
            spacing: 3,
        }
    }
}
