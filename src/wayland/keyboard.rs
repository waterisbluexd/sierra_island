use slint::{
    SharedString,
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

fn dispatch_key(
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
