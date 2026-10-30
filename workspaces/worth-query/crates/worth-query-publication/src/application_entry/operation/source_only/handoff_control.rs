//! A receiver-local rendezvous after genuine completion and before request handoff.
use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;

pub(super) struct CompletionHandoff {
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
/// Owns one receiver-local completion pause; dropping it releases only that pause.
pub struct WorthQuerySourceCompletionHandoffPauseForTest {
    receiver: Arc<CompletionHandoff>,
}
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Parks this operation only after actual native completion; no outcome or authority is supplied.
    #[doc(hidden)]
    pub fn pause_after_source_completion_for_test(
        &mut self,
    ) -> WorthQuerySourceCompletionHandoffPauseForTest {
        if let Some(previous) = &self.completion_handoff {
            assert!(
                !previous.armed.load(Ordering::Acquire),
                "one completion handoff pause may be armed"
            );
        }
        let receiver = Arc::new(CompletionHandoff {
            armed: AtomicBool::new(true),
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        });
        self.completion_handoff = Some(Arc::clone(&receiver));
        WorthQuerySourceCompletionHandoffPauseForTest { receiver }
    }
}
impl CompletionHandoff {
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
        let (mut state, expired) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(120), |state| !state.released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if expired.timed_out() && !state.released {
            state.released = true;
            self.changed.notify_all();
            drop(state);
            panic!("source completion handoff backstop expired");
        }
    }
}
impl WorthQuerySourceCompletionHandoffPauseForTest {
    /// Observes the real completion boundary without changing its product outcome.
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
    /// Releases this pause, including when the test did not reach its expected boundary.
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
impl Drop for WorthQuerySourceCompletionHandoffPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
