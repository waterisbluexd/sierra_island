use super::{behavior::Behavior, layout::Layout};

#[derive(Debug, Clone)]
pub struct Container {
    pub id: u32,
    pub name: String,
    pub layout: Layout,
    pub behavior: Behavior,
}
