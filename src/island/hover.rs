use layer_shika_adapters::AppState;

use super::surface::ISLAND;
use super::trigger;

pub fn is_anything_hovered(app_state: &mut AppState) -> bool {
    if trigger::is_hovered(app_state) {
        return true;
    }

    for surface in app_state.surfaces_by_name_mut(ISLAND) {
        if let Ok(value) = surface.component_instance().get_property("hovered") {
            if let Ok(hovered) = bool::try_from(value) {
                if hovered {
                    return true;
                }
            }
        }
    }

    false
}
