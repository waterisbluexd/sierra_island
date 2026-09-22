use slint::SharedPixelBuffer;
use slint::platform::software_renderer::PremultipliedRgbaColor;

use smithay_client_toolkit::shell::WaylandSurface;



pub trait SierraRenderer {
    fn draw(&mut self) -> bool;
    fn attach_buffer(&self, buffer: &smithay_client_toolkit::shm::slot::Buffer);
}

impl SierraRenderer for crate::wayland::state::SierraState {
    fn draw(&mut self) -> bool {
        let width = self.width;
        let height = self.height;

        let slint_window = self.slint_window.clone();

        let pool = &mut self.pool;

        let buffer_slot = &mut self.buffer;

        let rendered = slint_window.draw_if_needed(|renderer| {
            let mut slint_buffer = SharedPixelBuffer::<PremultipliedRgbaColor>::new(
                width,
                height,
            );

            renderer.render(slint_buffer.make_mut_slice(), width as usize);

            let buffer = buffer_slot.get_or_insert_with(|| {
                pool.create_buffer(
                    width as i32,
                    height as i32,
                    width as i32 * 4,
                    smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
                )
                .expect("Failed to create wl_shm buffer")
                .0
            });

            let canvas = match pool.canvas(buffer) {
                Some(canvas) => canvas,

                None => {
                    let (second_buffer, canvas) = pool
                        .create_buffer(
                            width as i32,
                            height as i32,
                            width as i32 * 4,
                            smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
                        )
                        .expect("Failed to create second wl_shm buffer");

                    *buffer = second_buffer;

                    canvas
                }
            };

            for (dst, src) in canvas
                .chunks_exact_mut(4)
                .zip(slint_buffer.as_slice().iter())
            {
                dst[0] = src.blue;
                dst[1] = src.green;
                dst[2] = src.red;
                dst[3] = src.alpha;
            }
        });

        if rendered {
            if let Some(buffer) = self.buffer.as_ref() {
                let _ = buffer.attach_to(self.layer.wl_surface());
                self.layer.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
                self.layer.wl_surface().commit();
            }
        }

        rendered
    }

    fn attach_buffer(&self, buffer: &smithay_client_toolkit::shm::slot::Buffer) {
        let _ = buffer.attach_to(self.layer.wl_surface());
        self.layer.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.layer.wl_surface().commit();
    }
}
