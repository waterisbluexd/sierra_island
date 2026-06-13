use crate::theme::Theme;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn apply_theme(instance: &ComponentInstance, theme: &Theme) {
    fn css_to_rgb8(hex: &str) -> layer_shika::slint::RgbaColor<u8> {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        layer_shika::slint::RgbaColor {
            red: r,
            green: g,
            blue: b,
            alpha: 255,
        }
    }

    let set = |prop: &str, hex: &str| {
        let color = layer_shika::slint::Color::from(css_to_rgb8(hex));
        if let Err(err) = instance.set_global_property("Theme", prop, Value::Brush(color.into())) {
            eprintln!("Failed to set Theme.{prop}: {err}");
        }
    };

    set("background", &theme.special.background);
    set("foreground", &theme.special.foreground);
    set("color0", &theme.colors.color0);
    set("color1", &theme.colors.color1);
    set("color2", &theme.colors.color2);
    set("color3", &theme.colors.color3);
    set("color4", &theme.colors.color4);
    set("color5", &theme.colors.color5);
    set("color6", &theme.colors.color6);
    set("color7", &theme.colors.color7);
    set("color8", &theme.colors.color8);
    set("color9", &theme.colors.color9);
    set("color10", &theme.colors.color10);
    set("color11", &theme.colors.color11);
    set("color12", &theme.colors.color12);
    set("color13", &theme.colors.color13);
    set("color14", &theme.colors.color14);
    set("color15", &theme.colors.color15);
}

/// Watches the filesystem and queues up newly parsed themes into shared memory
pub fn start_watcher(theme_queue: Arc<Mutex<Option<Theme>>>) {
    let theme_path = Theme::path();
    let (watcher_tx, watcher_rx) = mpsc::channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new(watcher_tx, Config::default()).expect("Failed to create watcher");

    let watch_dir = theme_path.parent().unwrap_or(&theme_path).to_path_buf();
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .expect("Failed to watch directory");

    thread::spawn(move || {
        for event in watcher_rx {
            if let Ok(event) = event {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                    && event.paths.iter().any(|p| p == &theme_path)
                {
                    println!("[Watcher] Pywal change noticed on disk.");
                    thread::sleep(Duration::from_millis(150)); // let pywal finish writing safely

                    let fresh_theme = Theme::load();

                    // Safely pass the new theme data to the shared slot
                    if let Ok(mut lock) = theme_queue.lock() {
                        *lock = Some(fresh_theme);
                        println!("[Watcher] Fresh theme safely queued for UI pickup!");
                    }
                }
            }
        }
    });

    std::mem::forget(watcher);
}
