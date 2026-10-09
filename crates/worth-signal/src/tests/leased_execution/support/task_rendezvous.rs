use std::sync::{Condvar, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct TaskRendezvous {
    state: Mutex<State>,
    wake: Condvar,
}

#[derive(Default)]
struct State {
    first: Option<ThreadId>,
    met: bool,
    expired: bool,
}

impl TaskRendezvous {
    pub(crate) fn meet(&self) {
        let mut state = self.state.lock().unwrap();
        if state.met || state.expired {
            return;
        }
        let thread = std::thread::current().id();
        if let Some(first) = state.first {
            assert_ne!(
                first, thread,
                "two task bodies must rendezvous on different workers"
            );
            state.met = true;
            self.wake.notify_all();
            return;
        }
        state.first = Some(thread);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !state.met {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                state.expired = true;
                return;
            };
            state = self.wake.wait_timeout(state, remaining).unwrap().0;
        }
    }

    pub(crate) fn assert_if_parallel(&self, resolved_parallel: bool) {
        let state = self.state.lock().unwrap();
        println!(
            "PLACEMENT_RENDEZVOUS resolved_parallel={resolved_parallel} met={} expired={}",
            state.met, state.expired
        );
        if resolved_parallel {
            assert!(
                state.met,
                "Automatic execution failed: two task bodies did not rendezvous within ten seconds"
            );
        }
    }
}
