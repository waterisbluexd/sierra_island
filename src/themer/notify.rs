use crate::theme::Theme;
use layer_shika::slint::{ComponentHandle, Weak};
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
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

    instance.window().request_redraw();
}

pub fn start_watcher(weak: Weak<ComponentInstance>) {
    let theme_path = Theme::path();
    let cache_dir: PathBuf = theme_path
        .parent()
        .and_then(|p| p.parent())
        .expect("Could not determine cache directory")
        .to_path_buf();

    println!("Watching recursively: {}", cache_dir.display());
    println!("Target file: {}", theme_path.display());

    let (tx, rx) = mpsc::channel();

    let mut watcher: RecommendedWatcher =
        Watcher::new(tx, Config::default()).expect("Failed to create watcher");

    watcher
        .watch(&cache_dir, RecursiveMode::Recursive)
        .expect("Failed to watch ~/.cache");

    thread::spawn(move || {
        let mut last_trigger = std::time::Instant::now() - Duration::from_secs(1);

        for event in rx {
            match event {
                Ok(event) => {
                    let relevant = event.paths.iter().any(|p| p == &theme_path);

                    if !relevant {
                        continue;
                    }

                    if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        continue;
                    }

                    let now = std::time::Instant::now();
                    if now.duration_since(last_trigger) < Duration::from_millis(150) {
                        continue;
                    }
                    last_trigger = now;

                    println!("[Watcher] colors.json changed, reloading");

                    let weak = weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        let new_theme = Theme::load();
                        if let Some(instance) = weak.upgrade() {
                            apply_theme(&instance, &new_theme);
                            println!("[Reload] Theme applied");
                        }
                    });
                }
                Err(e) => eprintln!("Watcher error: {}", e),
            }
        }
    });

    std::mem::forget(watcher);
}
