mod containers;
mod island;
mod themer;

pub mod widgets;

fn main() -> layer_shika::Result<()> {
    island::run()
}
