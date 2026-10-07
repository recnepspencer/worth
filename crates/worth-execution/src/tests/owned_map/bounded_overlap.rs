use std::{
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

/// Reusable overlap gate with a failure watchdog for missing admission.
/// Passing runs release on worker arrivals and never wait for the watchdog.
pub(super) struct Overlap {
    width: usize,
    state: Mutex<(usize, usize)>,
    changed: Condvar,
}
impl Overlap {
    pub(super) fn new(width: usize) -> Self {
        Self {
            width,
            state: Mutex::new((0, 0)),
            changed: Condvar::new(),
        }
    }
    pub(super) fn wait(&self) {
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut state = self.state.lock().unwrap();
        let generation = state.1;
        state.0 += 1;
        if state.0 == self.width {
            *state = (0, generation + 1);
            self.changed.notify_all();
            return;
        }
        while state.1 == generation {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let (next, timeout) = self.changed.wait_timeout(state, remaining).unwrap();
            state = next;
            assert!(
                !timeout.timed_out() || state.1 != generation,
                "requested worker overlap was not admitted"
            );
        }
    }
}
