use slint::{
    ComponentHandle, Model, SharedString,
    platform::{Key, WindowEvent},
};
use smithay_client_toolkit::reexports::client::protocol::{
    wl_keyboard::WlKeyboard, wl_surface::WlSurface,
};
use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
use smithay_client_toolkit::seat::keyboard::{KeyEvent, KeyboardHandler, Modifiers, RawModifiers};

use crate::wayland::state::SierraState;

impl KeyboardHandler for SierraState {
    fn enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym],
    ) {
        self.island_keyboard_entered = true;
        let _ = self
            .slint_window
            .try_dispatch_event(WindowEvent::WindowActiveChanged(true));
    }

    fn leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _surface: &WlSurface,
        _serial: u32,
    ) {
        self.island_keyboard_entered = false;
        let _ = self
            .slint_window
            .try_dispatch_event(WindowEvent::WindowActiveChanged(false));
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        match event.keysym.raw() {
            0xff54 if self.search_results.row_count() > 0 => {
                self.navigate_search_selection(1);
                return;
            }
            0xff52 if self.search_results.row_count() > 0 => {
                self.navigate_search_selection(-1);
                return;
            }
            0xff0d => {
                self.launch_selected_search();
                return;
            }
            _ => {}
        }
        dispatch_key(self, &event, |text| WindowEvent::KeyPressed { text });
    }

    fn repeat_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        match event.keysym.raw() {
            0xff54 if self.search_results.row_count() > 0 => {
                self.navigate_search_selection(1);
                return;
            }
            0xff52 if self.search_results.row_count() > 0 => {
                self.navigate_search_selection(-1);
                return;
            }
            _ => {}
        }
        dispatch_key(self, &event, |text| WindowEvent::KeyPressRepeated { text });
    }

    fn release_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        event: KeyEvent,
    ) {
        match event.keysym.raw() {
            0xff54 | 0xff52 | 0xff0d => return,
            _ => {}
        }
        dispatch_key(self, &event, |text| WindowEvent::KeyReleased { text });
    }

    fn update_modifiers(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _keyboard: &WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
    }
}

pub(super) fn dispatch_key(
    state: &SierraState,
    event: &KeyEvent,
    make_event: impl FnOnce(SharedString) -> WindowEvent,
) {
    if let Some(text) = key_text(event) {
        let _ = state
            .slint_window
            .try_dispatch_event(make_event(SharedString::from(text.as_str())));
    }
}

fn key_text(event: &KeyEvent) -> Option<String> {
    if let Some(utf8) = &event.utf8
        && !utf8.is_empty()
    {
        return Some(utf8.clone());
    }

    let char_code = match event.keysym.raw() {
        // Modifier keys
        0xffe1 => Key::Shift,
        0xffe2 => Key::ShiftR,
        0xffe3 => Key::Control,
        0xffe4 => Key::ControlR,
        0xffe7 => Key::Meta,
        0xffe8 => Key::MetaR,
        0xffe9 => Key::Alt,
        0xffea => Key::AltGr,
        // Navigation & editing
        0xff0d => Key::Return,
        0xff08 => Key::Backspace,
        0xff09 => Key::Tab,
        0xff07 => Key::Backtab,
        0xff1b => Key::Escape,
        0xffff => Key::Delete,
        0xff50 => Key::Home,
        0xff51 => Key::LeftArrow,
        0xff52 => Key::UpArrow,
        0xff53 => Key::RightArrow,
        0xff54 => Key::DownArrow,
        0xff55 => Key::PageUp,
        0xff56 => Key::PageDown,
        0xff57 => Key::End,
        _ => return event.keysym.key_char().map(|c| c.to_string()),
    };

    Some(char::from(char_code).to_string())
}

impl SierraState {
    fn navigate_search_selection(&mut self, direction: i32) {
        let search_nav = self.island.global::<crate::SearchNavigation>();
        let current = search_nav.get_selected_index();
        let results_len = self.search_results.row_count() as i32;

        if results_len <= 0 {
            return;
        }

        let new_index = if current < 0 || current >= results_len {
            if direction > 0 { 0 } else { results_len - 1 }
        } else {
            match direction {
                1 => (current + 1) % results_len,
                -1 => (current - 1 + results_len) % results_len,
                _ => current,
            }
        };

        search_nav.set_selected_index(new_index);

        let current_window = search_nav.get_window_start();
        let mut window_start = current_window;
        if new_index < window_start {
            window_start = new_index;
        } else if new_index >= window_start + 7 {
            window_start = new_index - 6;
        }
        let max_window = results_len.saturating_sub(7).max(0);
        window_start = window_start.clamp(0, max_window);
        search_nav.set_window_start(window_start);

        if let Some(name) = self.search_results.row_data(new_index as usize) {
            search_nav.set_selected_result(name);
        }
    }

    fn launch_selected_search(&mut self) {
        let search_nav = self.island.global::<crate::SearchNavigation>();
        let index = search_nav.get_selected_index();
        if index < 0 {
            return;
        }

        if index >= self.search_results_full.len() as i32 {
            return;
        }

        let name = self.search_results_full[index as usize].clone();

        let exec = {
            let discovery = self.discovery.borrow();
            discovery
                .get_application(name.as_str())
                .and_then(|app| app.exec.clone())
        };

        if let Some(exec) = exec {
            Self::launch_exec(&exec);
            self.dismiss_after_launch();
        }
    }

    fn launch_exec(exec: &str) {
        let exec = exec.replace("%u", "").replace("%f", "").replace("%c", "");
        let mut parts = exec.split_whitespace();
        if let Some(cmd) = parts.next() {
            let mut command = std::process::Command::new(cmd);
            for arg in parts {
                command.arg(arg);
            }
            let _ = command.spawn();
        }
    }
}
