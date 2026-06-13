mod theme;
mod themer;

use layer_shika::prelude::*;
use layer_shika_adapters::AppState;

const ISLAND: &str = "Island";

fn main() -> layer_shika::Result<()> {
    let mut shell = Shell::from_file("ui/island.slint")
        .surface(ISLAND)
        .width(300)
        .height(60)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(2)
        .build()?;

    shell.with_component(ISLAND, |instance| {
        let theme = theme::Theme::load();
        themer::apply_theme(instance, &theme);
        println!("Initial theme applied");
    });

    let loop_handle = shell.event_loop_handle();
    let (_token, sender) =
        loop_handle.add_channel::<(), _>(move |_msg, app_state: &mut AppState| {
            println!("[UI] Reloading theme...");
            let theme = theme::Theme::load();
            for surface in app_state.surfaces_by_name_mut(ISLAND) {
                themer::apply_theme(surface.component_instance(), &theme);
            }
            for surface in app_state.all_outputs() {
                if let Err(err) = surface.render_frame_if_dirty() {
                    eprintln!("Failed to render frame: {err}");
                }
            }
            println!("[UI] Theme applied.");
        })?;

    themer::start_watcher(sender);

    shell.run()?;
    Ok(())
}
