use crate::theme::Theme;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
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

/// Starts watching the theme file using OS notifications (inotify on Linux).
/// When a change is detected, it sets the `need_reload` atomic flag.
pub fn start_watcher_with_flag(need_reload: Arc<AtomicBool>) {
    let theme_path = Theme::path();
    println!("Watching for theme changes: {}", theme_path.display());

    let (watcher_tx, watcher_rx) = mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new(
        watcher_tx,
        Config::default().with_poll_interval(Duration::from_secs(2)),
    )
    .expect("Failed to create watcher");

    let watch_dir = theme_path.parent().unwrap_or(&theme_path).to_path_buf();
    println!("Watching directory: {}", watch_dir.display());
    watcher
        .watch(&watch_dir, RecursiveMode::NonRecursive)
        .expect("Failed to watch directory");

    thread::spawn(move || {
        for event in watcher_rx {
            match event {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                        && event
                            .paths
                            .iter()
                            .any(|p| p == &theme_path || p == &watch_dir)
                    {
                        println!("[Watcher] Change detected, setting reload flag");
                        need_reload.store(true, std::sync::atomic::Ordering::Relaxed);
                        // Brief delay to avoid multiple rapid notifications
                        thread::sleep(Duration::from_millis(50));
                    }
                }
                Err(e) => eprintln!("Watcher error: {}", e),
            }
        }
    });

    // Keep the watcher alive for the program's lifetime
    std::mem::forget(watcher);
}
