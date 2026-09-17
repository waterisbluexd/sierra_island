use layer_shika::prelude::*;
use layer_shika::slint_interpreter::Value;
use layer_shika_adapters::AppState;

pub const TRIGGER: &str = "Trigger";
pub const ISLAND: &str = "Island";

pub const TRIGGER_WIDTH: u32 = 260;
pub const TRIGGER_HEIGHT: u32 = 4;

pub const COLLAPSED_WIDTH: u32 = 250;
pub const COLLAPSED_HEIGHT: u32 = 1;

pub fn create_shell() -> layer_shika::Result<Shell> {
    Shell::from_file("ui/island.slint")
        // Invisible hover trigger
        .surface(TRIGGER)
        .width(TRIGGER_WIDTH)
        .height(TRIGGER_HEIGHT)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(0)
        // Actual Island surface
        .surface(ISLAND)
        .width(COLLAPSED_WIDTH)
        .height(COLLAPSED_HEIGHT)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(0)
        .build()
}

pub fn set_visible(app_state: &mut AppState, visible: bool) {
    for surface in app_state.surfaces_by_name_mut(ISLAND) {
        if let Err(err) = surface
            .component_instance()
            .set_property("visible-requested", Value::from(visible))
        {
            eprintln!("Failed to set Island visibility to {visible}: {err}");
        }
    }
}
