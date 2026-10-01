use std::time::Instant;

use slint::{LogicalPosition, platform::WindowEvent};

use smithay_client_toolkit::reexports::calloop::{
    timer::{TimeoutAction, Timer},
    LoopHandle,
};

use smithay_client_toolkit::shell::WaylandSurface;

use slint::platform::PointerEventButton;

use crate::wayland::state::{CLOSE_DELAY, COLLAPSE_DELAY, EventCommand, SierraState};

pub fn button_from_linux(button: u32) -> PointerEventButton {
    match button {
        0x110 => PointerEventButton::Left,
        0x111 => PointerEventButton::Right,
        0x112 => PointerEventButton::Middle,
        0x113 => PointerEventButton::Back,
        0x114 => PointerEventButton::Forward,
        _ => PointerEventButton::Other,
    }
}

impl smithay_client_toolkit::seat::pointer::PointerHandler for crate::wayland::state::SierraState {
    fn pointer_frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _pointer: &smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        let mut hover_changed = false;

        for event in events {
            let is_trigger = &event.surface == self.trigger_layer.wl_surface();

            let is_island = &event.surface == self.layer.wl_surface();

            if !is_trigger && !is_island {
                continue;
            }

            match event.kind {
                smithay_client_toolkit::seat::pointer::PointerEventKind::Enter { .. } => {
                    if is_trigger {
                        self.trigger_hovered = true;
                        hover_changed = true;
                    }

                    if is_island {
                        self.island_hovered = true;
                        hover_changed = true;

                        let position =
                            LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

                        let _ = self
                            .slint_window
                            .try_dispatch_event(WindowEvent::PointerMoved { position });
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Leave { .. } => {
                    if is_trigger {
                        self.trigger_hovered = false;
                        hover_changed = true;
                    }

                    if is_island {
                        self.island_hovered = false;
                        hover_changed = true;

                        let _ = self
                            .slint_window
                            .try_dispatch_event(WindowEvent::PointerExited);
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Motion { .. } => {
                    if is_island {
                        let position =
                            LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

                        let _ = self
                            .slint_window
                            .try_dispatch_event(WindowEvent::PointerMoved { position });
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Press {
                    button, ..
                } => {
                    if is_island {
                        let position =
                            LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

                        let _ = self
                            .slint_window
                            .try_dispatch_event(WindowEvent::PointerPressed {
                                position,
                                button: button_from_linux(button),
                            });
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Release {
                    button, ..
                } => {
                    if is_island {
                        let position =
                            LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

                        let _ =
                            self.slint_window
                                .try_dispatch_event(WindowEvent::PointerReleased {
                                    position,
                                    button: button_from_linux(button),
                                });
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    if is_island {
                        let position =
                            LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

                        let _ = self.slint_window.try_dispatch_event(WindowEvent::PointerScrolled {
                            position,
                            delta_x: horizontal.absolute as f32,
                            delta_y: vertical.absolute as f32,
                        });
                    }
                }
            }
        }

        if hover_changed {
            let hovered = self.trigger_hovered || self.island_hovered;
            let _ = self.command_sender.as_ref().map(|sender| {
                if hovered {
                    sender.send(crate::wayland::state::EventCommand::PointerEnter)
                } else {
                    sender.send(crate::wayland::state::EventCommand::PointerLeave)
                }
            });
        }
    }
}

pub fn handle_pointer_enter(
    state: &mut SierraState,
    loop_handle: &LoopHandle<'_, SierraState>,
) {
    state.close_timer_pending = false;
    state.collapse_timer_pending = false;

    if let Some(token) = state.close_timer_token.take() {
        let _ = loop_handle.remove(token);
    }
    if let Some(token) = state.collapse_timer_token.take() {
        let _ = loop_handle.remove(token);
    }

    state.show_island();
    slint::platform::update_timers_and_animations();
    let _ = state.draw();
}

pub fn handle_pointer_leave(
    state: &mut SierraState,
    loop_handle: &LoopHandle<'_, SierraState>,
) {
    state.update_hover(Instant::now());

    if state.close_timer_pending && state.close_timer_token.is_none() {
        let token = loop_handle.insert_source(
            Timer::from_duration(CLOSE_DELAY),
            |_, _, state| {
                state.close_timer_pending = false;
                state.close_timer_token = None;

                if !state.trigger_hovered
                    && !state.island_hovered
                    && state.island_expanded
                {
                    state.hide_island();
                    state.collapse_timer_pending = true;

                    slint::platform::update_timers_and_animations();
                    let _ = state.draw();

                    let _ = state.command_sender.as_ref().map(|sender| {
                        let _ = sender.send(EventCommand::ScheduleCollapseTimer);
                    });
                }

                TimeoutAction::Drop
            },
        );

        if let Ok(token) = token {
            state.close_timer_token = Some(token);
        }
    }

    slint::platform::update_timers_and_animations();
    let _ = state.draw();
}

pub fn schedule_collapse_timer(
    state: &mut SierraState,
    loop_handle: &LoopHandle<'_, SierraState>,
) {
    if state.collapse_timer_pending && state.collapse_timer_token.is_none() {
        let token = loop_handle.insert_source(
            Timer::from_duration(COLLAPSE_DELAY),
            |_, _, state| {
                state.collapse_timer_pending = false;
                state.collapse_timer_token = None;
                state.collapse_island();

                slint::platform::update_timers_and_animations();
                let _ = state.draw();

                TimeoutAction::Drop
            },
        );

        if let Ok(token) = token {
            state.collapse_timer_token = Some(token);
        }
    }
}
