use crate::theme::Theme as ThemeConfig;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use slint::ComponentHandle;
use smithay_client_toolkit::reexports::calloop::channel::Sender;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn css_to_color(hex: &str) -> slint::Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return slint::Color::from_rgb_u8(0, 0, 0);
    }

    let red = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let green = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let blue = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    slint::Color::from_rgb_u8(red, green, blue)
}

pub fn apply_theme(island: &crate::Island, theme: &ThemeConfig) {
    let global = island.global::<crate::Theme>();

    global.set_background(css_to_color(&theme.special.background).into());
    global.set_foreground(css_to_color(&theme.special.foreground).into());

    global.set_color0(css_to_color(&theme.colors.color0).into());
    global.set_color1(css_to_color(&theme.colors.color1).into());
    global.set_color2(css_to_color(&theme.colors.color2).into());
    global.set_color3(css_to_color(&theme.colors.color3).into());
    global.set_color4(css_to_color(&theme.colors.color4).into());
    global.set_color5(css_to_color(&theme.colors.color5).into());
    global.set_color6(css_to_color(&theme.colors.color6).into());
    global.set_color7(css_to_color(&theme.colors.color7).into());
    global.set_color8(css_to_color(&theme.colors.color8).into());
    global.set_color9(css_to_color(&theme.colors.color9).into());
    global.set_color10(css_to_color(&theme.colors.color10).into());
    global.set_color11(css_to_color(&theme.colors.color11).into());
    global.set_color12(css_to_color(&theme.colors.color12).into());
    global.set_color13(css_to_color(&theme.colors.color13).into());
    global.set_color14(css_to_color(&theme.colors.color14).into());
    global.set_color15(css_to_color(&theme.colors.color15).into());
}

pub fn start_watcher(sender: Sender<()>) {
    thread::spawn(move || {
        let theme_path = ThemeConfig::path();

        let (tx, rx) = mpsc::channel();

        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Config::default()).expect("Failed to create theme watcher");

        let watch_dir = theme_path.parent().unwrap_or(&theme_path).to_path_buf();

        watcher
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch theme directory");

        for event in rx {
            let Ok(event) = event else {
                continue;
            };

            let relevant = matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                && event.paths.iter().any(|path| path == &theme_path);

            if !relevant {
                continue;
            }

            thread::sleep(Duration::from_millis(150));

            if sender.send(()).is_err() {
                break;
            }
        }
    });
}
