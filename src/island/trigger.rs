use layer_shika_adapters::AppState;

use super::surface::TRIGGER;

pub fn is_hovered(app_state: &mut AppState) -> bool {
    for surface in app_state.surfaces_by_name_mut(TRIGGER) {
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
