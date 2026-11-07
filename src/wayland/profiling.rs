//! Profiling instrumentation for Sierra Island.
//!
//! This module is only active when the `profiling` Cargo feature is enabled.
//! When disabled, all types are zero-size no-ops with no runtime cost.

#[cfg(feature = "profiling")]
use std::collections::HashMap;
#[cfg(feature = "profiling")]
use std::time::Instant;

/// A timer that measures elapsed nanoseconds.
///
/// When profiling is disabled, this is a zero-size type.
pub struct Timer {
    #[cfg(feature = "profiling")]
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        #[cfg(feature = "profiling")]
        let start = Instant::now();
        Timer {
            #[cfg(feature = "profiling")]
            start,
        }
    }

    pub fn elapsed_ns(&self) -> u64 {
        #[cfg(feature = "profiling")]
        {
            return self.start.elapsed().as_nanos() as u64;
        }
        #[cfg(not(feature = "profiling"))]
        {
            0
        }
    }
}

/// Accumulates timing statistics (count, total, max).
pub struct Accumulator {
    #[cfg(feature = "profiling")]
    count: u64,
    #[cfg(feature = "profiling")]
    total_ns: u64,
    #[cfg(feature = "profiling")]
    max_ns: u64,
}

impl Accumulator {
    pub fn new() -> Self {
        Accumulator {
            #[cfg(feature = "profiling")]
            count: 0,
            #[cfg(feature = "profiling")]
            total_ns: 0,
            #[cfg(feature = "profiling")]
            max_ns: 0,
        }
    }

    pub fn add(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.count += 1;
            self.total_ns += _ns;
            if _ns > self.max_ns {
                self.max_ns = _ns;
            }
        }
    }

    #[allow(dead_code)]
    pub fn avg_ns(&self) -> u64 {
        #[cfg(feature = "profiling")]
        {
            if self.count == 0 {
                0
            } else {
                self.total_ns / self.count
            }
        }
        #[cfg(not(feature = "profiling"))]
        {
            0
        }
    }
}

/// All profiling statistics, aggregated across the lifetime of the application.
pub struct Stats {
    /// Every `draw()` invocation (timer wakeup that calls draw).
    pub draw_invocations: Accumulator,
    /// Every `draw_if_needed()` call (Slint render attempt).
    pub slint_render_attempts: Accumulator,
    /// Time Slint actually rendered new pixels (draw_if_needed returned true).
    pub actual_render_duration: Accumulator,
    /// Time between consecutive actual renders (frame interval).
    pub frame_interval: Accumulator,
    /// Pixel copy operations (RGBA→BGRA conversion).
    pub pixel_copy_ops: Accumulator,
    /// Wayland buffer submissions (attach + damage + commit).
    pub buffer_submissions: Accumulator,
    /// Search result processing (only when results actually received).
    pub search_processing: Accumulator,
    /// Search results actually received (non-empty try_recv).
    pub search_results_received: u64,
    /// Timer wakeups (animation + clock timers).
    pub timer_wakeups: u64,
    /// Slint render attempts that were skipped (no new content).
    pub slint_render_skipped: u64,
    /// Actual rendered frames (draw_if_needed returned true).
    pub rendered_frames: u64,
    /// Wayland buffer submissions count.
    pub buffer_submission_count: u64,
    #[cfg(feature = "profiling")]
    pub last_print: Option<Instant>,
    #[cfg(feature = "profiling")]
    pub last_render_time: Option<Instant>,
    #[cfg(feature = "profiling")]
    pub startup_stages: HashMap<&'static str, u64>,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            draw_invocations: Accumulator::new(),
            slint_render_attempts: Accumulator::new(),
            actual_render_duration: Accumulator::new(),
            frame_interval: Accumulator::new(),
            pixel_copy_ops: Accumulator::new(),
            buffer_submissions: Accumulator::new(),
            search_processing: Accumulator::new(),
            search_results_received: 0,
            timer_wakeups: 0,
            slint_render_skipped: 0,
            rendered_frames: 0,
            buffer_submission_count: 0,
            #[cfg(feature = "profiling")]
            last_print: None,
            #[cfg(feature = "profiling")]
            last_render_time: None,
            #[cfg(feature = "profiling")]
            startup_stages: HashMap::new(),
        }
    }

    pub fn record_startup(&mut self, _stage: &'static str, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            *self.startup_stages.entry(_stage).or_insert(0) += _ns;
        }
    }

    pub fn record_render(&mut self, _render_ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.actual_render_duration.add(_render_ns);
            self.rendered_frames += 1;
            if let Some(last) = self.last_render_time {
                self.frame_interval.add(last.elapsed().as_nanos() as u64);
            }
            self.last_render_time = Some(Instant::now());
        }
    }

    pub fn record_skip(&mut self) {
        #[cfg(feature = "profiling")]
        {
            self.slint_render_skipped += 1;
        }
    }

    pub fn record_buffer_submission(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.buffer_submissions.add(_ns);
            self.buffer_submission_count += 1;
        }
    }

    pub fn record_first_render(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.startup_stages
                .entry("first_frame_rendered")
                .or_insert(_ns);
        }
    }

    pub fn record_first_buffer_alloc(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.startup_stages
                .entry("first_buffer_alloc")
                .or_insert(_ns);
        }
    }

    pub fn record_first_buffer_submit(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.startup_stages
                .entry("first_buffer_submit")
                .or_insert(_ns);
        }
    }

    pub fn record_layer_surface_create(&mut self, _ns: u64) {
        #[cfg(feature = "profiling")]
        {
            self.startup_stages
                .entry("layer_surface_create")
                .or_insert(_ns);
        }
    }

    pub fn print_summary(&mut self, _force: bool) {
        #[cfg(feature = "profiling")]
        {
            let now = Instant::now();
            let should_print = _force
                || self
                    .last_print
                    .map(|lp| now.duration_since(lp) >= std::time::Duration::from_secs(5))
                    .unwrap_or(true);

            if !should_print {
                return;
            }
            self.last_print = Some(now);

            eprintln!("\n=== Sierra Island Profiling Summary ===");
            eprintln!(
                "Draw invocations: {} (avg {} ns, max {} ns)",
                self.draw_invocations.count,
                self.draw_invocations.avg_ns(),
                self.draw_invocations.max_ns
            );
            eprintln!(
                "Slint render attempts: {} (avg {} ns, max {} ns)",
                self.slint_render_attempts.count,
                self.slint_render_attempts.avg_ns(),
                self.slint_render_attempts.max_ns
            );
            eprintln!("Skipped renders: {}", self.slint_render_skipped);
            eprintln!("Actual rendered frames: {}", self.rendered_frames);
            eprintln!(
                "Actual render duration: {} (avg {} ns, max {} ns)",
                self.actual_render_duration.count,
                self.actual_render_duration.avg_ns(),
                self.actual_render_duration.max_ns
            );
            eprintln!(
                "Frame interval: {} (avg {} ns, max {} ns)",
                self.frame_interval.count,
                self.frame_interval.avg_ns(),
                self.frame_interval.max_ns
            );
            eprintln!(
                "Pixel copy ops: {} (avg {} ns, max {} ns)",
                self.pixel_copy_ops.count,
                self.pixel_copy_ops.avg_ns(),
                self.pixel_copy_ops.max_ns
            );
            eprintln!(
                "Buffer submissions: {} (avg {} ns, max {} ns)",
                self.buffer_submission_count,
                self.buffer_submissions.avg_ns(),
                self.buffer_submissions.max_ns
            );
            eprintln!("Search results received: {}", self.search_results_received);
            eprintln!(
                "Search processing: {} (avg {} ns, max {} ns)",
                self.search_processing.count,
                self.search_processing.avg_ns(),
                self.search_processing.max_ns
            );
            eprintln!("Timer wakeups: {}", self.timer_wakeups);
            eprintln!("Startup stages:");
            for (stage, ns) in &self.startup_stages {
                eprintln!(
                    "  {}: {} ns ({:.2} ms)",
                    stage,
                    *ns,
                    *ns as f64 / 1_000_000.0
                );
            }
            eprintln!("========================================\n");
        }
    }
}
