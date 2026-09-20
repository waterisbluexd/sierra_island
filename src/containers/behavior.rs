#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverAction {
    None,
    Show,
    Hide,
    Expand,
    Collapse,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    None,
    Show,
    Hide,
    Expand,
    Collapse,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoubleClickAction {
    None,
    Show,
    Hide,
    Expand,
    Collapse,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Behavior {
    pub hover: HoverAction,
    pub click: ClickAction,
    pub double_click: DoubleClickAction,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            hover: HoverAction::None,
            click: ClickAction::None,
            double_click: DoubleClickAction::None,
        }
    }
}
