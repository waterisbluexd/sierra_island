slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let island = Island::new()?;
    island.run()
}
