use layer_shika_adapters::AppState;

pub fn render_dirty(app_state: &mut AppState) {
    for surface in app_state.all_outputs() {
        if let Err(err) = surface.render_frame_if_dirty() {
            eprintln!("Failed to render frame: {err}");
        }
    }
}
