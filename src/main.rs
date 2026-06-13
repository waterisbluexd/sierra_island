mod theme;
mod themer;

use layer_shika::slint_interpreter::{ComponentHandle, Value};
use layer_shika::{ShellRuntime, prelude::*};
use std::sync::{Arc, Mutex};

fn main() -> layer_shika::Result<()> {
    let mut shell = Shell::from_file("ui/island.slint")
        .surface("Island")
        .width(300)
        .height(60)
        .anchor(AnchorEdges::empty().with_top())
        .exclusive_zone(0)
        .margin(2)
        .build()?;

    // Thread-safe communication slot
    let theme_queue = Arc::new(Mutex::new(None));
    let theme_queue_ui = theme_queue.clone();

    shell.with_component("Island", |instance| {
        // Load default layout colors on boot
        let theme = theme::Theme::load();
        themer::notify::apply_theme(instance, &theme);
        println!("Initial theme applied");

        let weak_instance = instance.as_weak();
        let queue = theme_queue_ui.clone();

        // Link the Slint internal timer callback directly to our shared cache check
        let res = instance.set_global_callback("ThemeWatcher", "check_updates", move |_args| {
            // Using non-blocking try_lock so the UI thread never hitches
            if let Ok(mut lock) = queue.try_lock() {
                if let Some(fresh_theme) = lock.take() {
                    if let Some(ui) = weak_instance.upgrade() {
                        themer::notify::apply_theme(&ui, &fresh_theme);
                        println!("[UI Thread] Polled and successfully updated live Pywal theme!");
                    }
                }
            }
            Value::Void
        });

        if let Err(e) = res {
            eprintln!("Failed to register ThemeWatcher loop hook: {:?}", e);
        }
    });

    // Start background filesystem watcher thread
    themer::start_watcher(theme_queue);

    shell.run()?;
    Ok(())
}
