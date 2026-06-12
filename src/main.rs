mod theme;
mod themer;

use layer_shika::slint::ComponentHandle;
use layer_shika::{ShellRuntime, prelude::*, slint};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
        let theme = theme::Theme::load();
        themer::notify::apply_theme(instance, &theme);
        println!("Initial theme applied");
        weak = Some(instance.as_weak());
    });

    let weak = weak.expect("Island component not found");
    let need_reload = Arc::new(AtomicBool::new(false));

    // Start the file watcher, passing the flag
    let watcher_flag = need_reload.clone();
    themer::notify::start_watcher_with_flag(watcher_flag);

    // Timer on the main event loop checks the flag every 100ms
    let weak_timer = weak.clone();
    let timer_flag = need_reload.clone();
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            if timer_flag.load(Ordering::Relaxed) {
                timer_flag.store(false, Ordering::Relaxed);
                println!("[Timer] Reloading theme due to file change");
                let new_theme = theme::Theme::load();
                if let Some(instance) = weak_timer.upgrade() {
                    themer::notify::apply_theme(&instance, &new_theme);
                    println!("[Timer] Theme reloaded and applied");
                }
            }
        },
    );

    // Keep the timer alive for the program's duration
    std::mem::forget(timer);

    shell.run()?;
    Ok(())
}
