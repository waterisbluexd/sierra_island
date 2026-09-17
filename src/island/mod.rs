mod animation;
mod controller;
mod hover;
mod rendering;
mod sizing;
mod surface;
mod trigger;

pub use controller::IslandController;

pub fn run() -> layer_shika::Result<()> {
    IslandController::new().run()
}
