mod theme;
mod themer;

use layer_shika::prelude::*;
use layer_shika::slint::ComponentHandle;

fn main() -> layer_shika::Result<()> {
    let mut shell = Shell::from_file("ui/island.slint")
        .surface("Island")
        .width(300)
        .height(60)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(2)
        .build()?;

    shell.with_component("Island", |instance| {
        let theme = theme::Theme::load();
        themer::apply_theme(instance, &theme);
        println!("Initial theme applied");

        themer::start_watcher(instance.as_weak());
    });

    shell.run()?;
    Ok(())
}
