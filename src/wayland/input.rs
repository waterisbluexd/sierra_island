use slint::{
    LogicalPosition,
    platform::WindowEvent,
};

use smithay_client_toolkit::shell::WaylandSurface;

use crate::wayland::clock::button_from_linux;

impl smithay_client_toolkit::seat::pointer::PointerHandler
    for crate::wayland::state::SierraState
{
    fn pointer_frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _pointer: &smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            let is_trigger =
                &event.surface == self.trigger_layer.wl_surface();

            let is_island =
                &event.surface == self.layer.wl_surface();

            if !is_trigger && !is_island {
                continue;
            }

            match event.kind {
                smithay_client_toolkit::seat::pointer::PointerEventKind::Enter { .. } => {
                    if is_trigger {
                        self.trigger_hovered = true;
                    }

                    if is_island {
                        self.island_hovered = true;

                        let position = LogicalPosition::new(
                            event.position.0 as f32,
                            event.position.1 as f32,
                        );

                        let _ = self
                            .slint_window
                            .try_dispatch_event(
                                WindowEvent::PointerMoved {
                                    position,
                                },
                            );
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Leave { .. } => {
                    if is_trigger {
                        self.trigger_hovered = false;
                    }

                    if is_island {
                        self.island_hovered = false;

                        let _ = self
                            .slint_window
                            .try_dispatch_event(
                                WindowEvent::PointerExited,
                            );
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Motion { .. } => {
                    if is_island {
                        let position = LogicalPosition::new(
                            event.position.0 as f32,
                            event.position.1 as f32,
                        );

                        let _ = self
                            .slint_window
                            .try_dispatch_event(
                                WindowEvent::PointerMoved {
                                    position,
                                },
                            );
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Press {
                    button,
                    ..
                } => {
                    if is_island {
                        let position = LogicalPosition::new(
                            event.position.0 as f32,
                            event.position.1 as f32,
                        );

                        let _ = self
                            .slint_window
                            .try_dispatch_event(
                                WindowEvent::PointerPressed {
                                    position,
                                    button: button_from_linux(button),
                                },
                            );
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Release {
                    button,
                    ..
                } => {
                    if is_island {
                        let position = LogicalPosition::new(
                            event.position.0 as f32,
                            event.position.1 as f32,
                        );

                        let _ = self
                            .slint_window
                            .try_dispatch_event(
                                WindowEvent::PointerReleased {
                                    position,
                                    button: button_from_linux(button),
                                },
                            );
                    }
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Axis { .. } => {}
            }
        }
    }
}
