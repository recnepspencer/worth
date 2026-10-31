//! Shared rendezvous mechanics for controls attached to distinct real document receivers.
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct ReceiverPause {
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Default)]
struct State {
    reached: bool,
    released: bool,
}
pub(super) struct ReceiverPauseGuard {
    receiver: Arc<ReceiverPause>,
}
impl ReceiverPauseGuard {
    pub(super) fn arm(slot: &mut Option<Arc<ReceiverPause>>) -> Self {
        if let Some(previous) = slot {
            assert!(
                previous
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .released,
                "one receiver pause may be armed"
            );
        }
        let receiver = Arc::new(ReceiverPause::default());
        *slot = Some(Arc::clone(&receiver));
        Self { receiver }
    }
    pub(super) fn wait_until_reached(&self, timeout: Duration) -> bool {
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
    pub(super) fn release(&self) {
        self.receiver
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .released = true;
        self.receiver.changed.notify_all();
    }
}
impl Drop for ReceiverPauseGuard {
    fn drop(&mut self) {
        self.release();
    }
}
impl ReceiverPause {
    pub(in crate::domain_computation::primary_graph) fn pause(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.released {
            return;
        }
        state.reached = true;
        self.changed.notify_all();
        let (state, expired) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(120), |state| !state.released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if expired.timed_out() && !state.released {
            drop(state);
            panic!("history receiver handoff backstop expired");
        }
    }
}
