use slint::platform::WindowEvent;
use slint::LogicalPosition;

use smithay_client_toolkit::shell::WaylandSurface;

use crate::wayland::clock::button_from_linux;

impl smithay_client_toolkit::seat::pointer::PointerHandler for crate::wayland::state::SierraState {
    fn pointer_frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _pointer: &smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }

            let position = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);

            match event.kind {
                smithay_client_toolkit::seat::pointer::PointerEventKind::Enter { .. } => {
                    println!("[INPUT] Pointer entered: {:?}", event.position);

                    let _ = self
                        .slint_window
                        .try_dispatch_event(WindowEvent::PointerMoved { position });
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Leave { .. } => {
                    println!("[INPUT] Pointer left");

                    let _ = self
                        .slint_window
                        .try_dispatch_event(WindowEvent::PointerExited);
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Motion { .. } => {
                    let _ = self
                        .slint_window
                        .try_dispatch_event(WindowEvent::PointerMoved { position });
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Press { button, .. } => {
                    let _ = self
                        .slint_window
                        .try_dispatch_event(WindowEvent::PointerPressed {
                            position,
                            button: button_from_linux(button),
                        });
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Release { button, .. } => {
                    let _ = self
                        .slint_window
                        .try_dispatch_event(WindowEvent::PointerReleased {
                            position,
                            button: button_from_linux(button),
                        });
                }

                smithay_client_toolkit::seat::pointer::PointerEventKind::Axis { .. } => {}
            }
        }
    }
}
