use super::theme::Theme;
use layer_shika::slint;
use layer_shika::slint_interpreter::{ComponentInstance, Value};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::thread;

fn apply_theme(instance: &ComponentInstance, theme: &Theme) {
    let set = |prop: &str, hex: &str| {
        let color = slint::Color::from(css_to_rgb8(hex));
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

fn css_to_rgb8(hex: &str) -> slint::RgbaColor<u8> {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    slint::RgbaColor {
        red: r,
        green: g,
        blue: b,
        alpha: 255,
    }
}

pub type WeakInstance = slint::Weak<layer_shika::slint_interpreter::ComponentInstance>;

pub fn start_watching(weak: WeakInstance) {
    {
        let theme = Theme::load();
        let weak = weak.clone();
        let result = weak.upgrade_in_event_loop(move |instance| {
            println!("Applying initial theme");
            apply_theme(&instance, &theme);
        });
        if result.is_err() {
            eprintln!("upgrade_in_event_loop failed at startup — instance not ready");
        }
    }

    thread::spawn(move || {
        let path = Theme::path();

        println!("Watching: {}", path.display());

        let weak_for_watcher = weak.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        println!("Pywal theme changed, reloading");

                        let theme = Theme::load();
                        let weak = weak_for_watcher.clone();

                        weak.upgrade_in_event_loop(move |instance| {
                            apply_theme(&instance, &theme);
                        })
                        .ok();
                    }
                }

                Err(err) => {
                    eprintln!("Watcher error: {err}");
                }
            })
            .unwrap();

        let watch_dir = path.parent().unwrap_or(&path);
        watcher
            .watch(watch_dir, RecursiveMode::NonRecursive)
            .unwrap();

        loop {
            thread::park();
        }
    });
}
