use slint::SharedPixelBuffer;

use smithay_client_toolkit::shell::WaylandSurface;

pub trait SierraRenderer {
    fn draw(&mut self) -> bool;
}

impl SierraRenderer for crate::wayland::state::SierraState {
    fn draw(&mut self) -> bool {
        if !self.configured {
            return false;
        }

        let width = self.width;
        let height = self.height;

        let slint_window = self.slint_window.clone();

        let pool = &mut self.pool;
        let surfaces = &mut self.surfaces;
        let slint_buffer = &mut self.slint_buffer;

        // Render the island once, then copy the pixels into every output.
        let slint_render_timer = crate::wayland::profiling::Timer::start();
        let rendered = slint_window.draw_if_needed(|renderer| {
            if slint_buffer.as_ref().is_some_and(|buf| {
                buf.width() != width || buf.height() != height
            }) {
                slint_buffer.take();
            }

            let mut buf = slint_buffer.take().unwrap_or_else(|| {
                SharedPixelBuffer::new(width, height)
            });

            renderer.render(
                buf.make_mut_slice(),
                width as usize,
            );

            for surface in surfaces.iter_mut().filter(|s| s.configured && s.active) {
                let first_alloc = surface.buffer.is_none();
                let buffer = surface.buffer.get_or_insert_with(|| {
                    pool.create_buffer(
                        width as i32,
                        height as i32,
                        width as i32 * 4,
                        smithay_client_toolkit::reexports::client::protocol::wl_shm::Format::Argb8888,
                    )
                    .expect("Failed to create wl_shm buffer")
                    .0
                });

                if first_alloc {
                    self.prof.record_first_buffer_alloc(
                        slint_render_timer.elapsed_ns(),
                    );
                }

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

                let copy_timer = crate::wayland::profiling::Timer::start();
                for (dst, src) in canvas
                    .as_chunks_mut::<4>().0.iter_mut()
                    .zip(buf.as_slice().iter())
                {
                    dst[0] = src.blue;
                    dst[1] = src.green;
                    dst[2] = src.red;
                    dst[3] = src.alpha;
                }
                self.prof.pixel_copy_ops.add(copy_timer.elapsed_ns());
            }

            *slint_buffer = Some(buf);
        });
        self.prof
            .slint_render_attempts
            .add(slint_render_timer.elapsed_ns());

        if rendered {
            self.prof.record_render(slint_render_timer.elapsed_ns());
        }

        if !rendered {
            return false;
        }

        let commit_timer = crate::wayland::profiling::Timer::start();
        let mut first_submit = false;
        for surface in self.surfaces.iter().filter(|s| s.configured && s.active) {
            let Some(buffer) = surface.buffer.as_ref() else {
                continue;
            };

            if self.prof.buffer_submission_count == 0 {
                first_submit = true;
            }

            buffer
                .attach_to(surface.layer.wl_surface())
                .expect("Failed to attach wl_shm buffer");

            surface
                .layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);

            surface.layer.commit();
        }
        self.prof
            .record_buffer_submission(commit_timer.elapsed_ns());
        if first_submit {
            self.prof
                .record_first_buffer_submit(commit_timer.elapsed_ns());
        }

        true
    }
}
