//! Receiver-local rendezvous after genuine adoption, before checked delivery returns.
use super::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;

pub(super) struct AdoptionHandoff {
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
pub struct WorthQueryHistoryEditHandoffPauseForTest {
    receiver: Arc<AdoptionHandoff>,
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
    > WorthQueryHistoryEdit<'_, S, H, I, P, A>
where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    #[doc(hidden)]
    pub fn pause_after_history_adoption_for_test(
        &mut self,
    ) -> WorthQueryHistoryEditHandoffPauseForTest {
        if let Some(previous) = &self.handoff {
            assert!(
                !previous.armed.load(Ordering::Acquire),
                "one adoption handoff pause may be armed"
            );
        }
        let receiver = Arc::new(AdoptionHandoff {
            armed: AtomicBool::new(true),
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
        });
        self.handoff = Some(Arc::clone(&receiver));
        WorthQueryHistoryEditHandoffPauseForTest { receiver }
    }
}

impl AdoptionHandoff {
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
            panic!("history adoption handoff backstop expired");
        }
    }
}

impl WorthQueryHistoryEditHandoffPauseForTest {
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
impl Drop for WorthQueryHistoryEditHandoffPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
