mod themer;

slint::include_modules!();

use layer_shika::prelude::*;

fn main() -> layer_shika::Result<()> {
    themer::notify::start_watching();

    Shell::from_file("ui/island.slint")
        .surface("Island")
        .width(300)
        .height(60)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(2)
        .build()?
        .run()?;

    Ok(())
}
