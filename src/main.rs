mod containers;
mod theme;
mod themer;
mod wayland;
mod widgets;

slint::include_modules!();

fn main() {
    wayland::run();
}
