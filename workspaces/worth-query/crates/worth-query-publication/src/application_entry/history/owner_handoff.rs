//! One receiver-local rendezvous at an actual managed-owner handoff.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;

pub(super) struct OwnerHandoff {
    armed: AtomicBool,
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Default)]
struct State {
    reached: bool,
    released: bool,
}

#[doc(hidden)]
pub struct WorthQueryHistoryOwnerHandoffPauseForTest {
    receiver: Arc<OwnerHandoff>,
}

impl OwnerHandoff {
    pub(super) fn arm(
        previous: Option<&Arc<Self>>,
    ) -> (Arc<Self>, WorthQueryHistoryOwnerHandoffPauseForTest) {
        if let Some(previous) = previous {
            assert!(
                !previous.armed.load(Ordering::Acquire),
                "one owner handoff pause may be armed"
            );
        }
        let receiver = Arc::new(Self {
            armed: AtomicBool::new(true),
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        });
        let control = WorthQueryHistoryOwnerHandoffPauseForTest {
            receiver: Arc::clone(&receiver),
        };
        (receiver, control)
    }
    pub(super) fn pause(&self) {
        if !self.armed.swap(false, Ordering::AcqRel) {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reached = true;
        self.changed.notify_all();
        let (state, expired) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(120), |state| !state.released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if expired.timed_out() && !state.released {
            drop(state);
            panic!("history owner handoff backstop expired");
        }
    }
}

impl WorthQueryHistoryOwnerHandoffPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self
            .receiver
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (state, _) = self
            .receiver
            .changed
            .wait_timeout_while(state, timeout, |state| !state.reached && !state.released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reached
    }
    pub fn release(&self) {
        self.receiver.armed.store(false, Ordering::Release);
        self.receiver
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .released = true;
        self.receiver.changed.notify_all();
    }
}
impl Drop for WorthQueryHistoryOwnerHandoffPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
