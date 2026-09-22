mod containers;
mod theme;
mod themer;
mod widgets;
mod wayland;

slint::include_modules!();

fn main() {
    wayland::run();
}
