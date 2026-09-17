use layer_shika_adapters::AppState;

use super::surface::{COLLAPSED_HEIGHT, COLLAPSED_WIDTH, ISLAND};

pub fn read_content_size(
    instance: &layer_shika::slint_interpreter::ComponentInstance,
) -> Option<(u32, u32)> {
    let width_value = instance.get_property("content-width").ok()?;
    let height_value = instance.get_property("content-height").ok()?;

    let width = f32::try_from(width_value).ok()? as u32;
    let height = f32::try_from(height_value).ok()? as u32;

    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

pub fn get_content_size(app_state: &mut AppState) -> Option<(u32, u32)> {
    for surface in app_state.surfaces_by_name_mut(ISLAND) {
        return read_content_size(surface.component_instance());
    }

    None
}

pub fn collapsed_size() -> (u32, u32) {
    (COLLAPSED_WIDTH, COLLAPSED_HEIGHT)
}
