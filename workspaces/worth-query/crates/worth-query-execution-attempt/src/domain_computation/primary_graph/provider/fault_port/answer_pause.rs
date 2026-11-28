//! One-shot certification pause gate right after commit resolution and before
//! the answer is returned to the caller.
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Default)]
struct State {
    reached: bool,
    released: bool,
}

#[derive(Clone, Default)]
pub struct WorthQueryCommitAnswerPauseForTest {
    inner: Arc<(Mutex<State>, Condvar)>,
}

impl WorthQueryCommitAnswerPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let (lock, condvar) = &*self.inner;
        let deadline = worth_execution::MonotonicClock::system().now() + timeout;
        let mut guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        while !guard.reached {
            let Some(remaining) =
                deadline.checked_duration_since(worth_execution::MonotonicClock::system().now())
            else {
                return false;
            };
            let (next, result) = condvar
                .wait_timeout(guard, remaining)
                .unwrap_or_else(|p| p.into_inner());
            guard = next;
            if result.timed_out() && !guard.reached {
                return false;
            }
        }
        true
    }

    pub fn release(&self) {
        let (lock, condvar) = &*self.inner;
        let mut guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        guard.released = true;
        condvar.notify_all();
    }

    pub(super) fn park(&self) {
        let (lock, condvar) = &*self.inner;
        let mut guard = lock.lock().unwrap_or_else(|p| p.into_inner());
        guard.reached = true;
        condvar.notify_all();
        while !guard.released {
            guard = condvar.wait(guard).unwrap_or_else(|p| p.into_inner());
        }
    }
}

#[derive(Default)]
pub(super) struct BoundaryAnswerPause {
    pending: Mutex<Option<WorthQueryCommitAnswerPauseForTest>>,
}

impl BoundaryAnswerPause {
    pub(super) fn arm(&self) -> WorthQueryCommitAnswerPauseForTest {
        let gate = WorthQueryCommitAnswerPauseForTest::default();
        let mut pending = self.pending.lock().unwrap_or_else(|p| p.into_inner());
        *pending = Some(gate.clone());
        gate
    }

    pub(super) fn pause_if_armed(&self) {
        let gate = self
            .pending
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        if let Some(gate) = gate {
            gate.park();
        }
    }
}
