mod themer;

use layer_shika::ShellRuntime;
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

    let mut weak = None;
    shell.with_component("Island", |instance| {
        weak = Some(instance.as_weak());
    });

    let weak = weak.expect("Island component not found");

    themer::notify::start_watching(weak);

    shell.run()?;

    Ok(())
}
