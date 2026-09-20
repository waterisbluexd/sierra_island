use std::time::{Duration, Instant};

pub const CLOSE_DELAY: Duration = Duration::from_millis(2000);
pub const COLLAPSE_DELAY: Duration = Duration::from_millis(220);

pub struct AnimationState {
    pub is_open: bool,
    pub hide_at: Option<Instant>,
    pub shrink_at: Option<Instant>,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            hide_at: None,
            shrink_at: None,
        }
    }

    pub fn cancel_close(&mut self) {
        self.hide_at = None;
    }

    pub fn schedule_close(&mut self, now: Instant) {
        if self.hide_at.is_none() {
            self.hide_at = Some(now + CLOSE_DELAY);
        }
    }

    pub fn should_close(&self, now: Instant) -> bool {
        self.hide_at.is_some_and(|at| now >= at)
    }

    pub fn mark_closed(&mut self, now: Instant) {
        self.hide_at = None;
        self.is_open = false;
        self.shrink_at = Some(now + COLLAPSE_DELAY);
    }

    pub fn should_collapse(&self, now: Instant) -> bool {
        self.shrink_at.is_some_and(|at| now >= at)
    }

    pub fn mark_collapsed(&mut self) {
        self.shrink_at = None;
    }
}
