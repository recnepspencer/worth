use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

#[derive(Clone)]
pub struct TestConcurrencyProbe {
    expected_entries: usize,
    state: Arc<(Mutex<ConcurrencyProbeState>, Condvar)>,
}

#[derive(Default)]
struct ConcurrencyProbeState {
    active_entries: usize,
    maximum_active_entries: usize,
    released: bool,
}

impl TestConcurrencyProbe {
    pub fn expecting(expected_entries: usize) -> Self {
        Self {
            expected_entries,
            state: Arc::new((Mutex::new(ConcurrencyProbeState::default()), Condvar::new())),
        }
    }

    pub fn maximum_active_entries(&self) -> usize {
        self.state
            .0
            .lock()
            .expect("concurrency probe lock")
            .maximum_active_entries
    }

    pub(super) fn enter_transaction(&self) -> bool {
        let (state_lock, changed) = &*self.state;
        let mut state = state_lock.lock().expect("concurrency probe lock");
        state.active_entries += 1;
        state.maximum_active_entries = state.maximum_active_entries.max(state.active_entries);
        if state.active_entries >= self.expected_entries {
            state.released = true;
            changed.notify_all();
        }
        let (mut state, timeout) = changed
            .wait_timeout_while(state, Duration::from_secs(2), |state| !state.released)
            .expect("concurrency probe wait");
        let admitted = !timeout.timed_out();
        state.active_entries -= 1;
        admitted
    }
}
