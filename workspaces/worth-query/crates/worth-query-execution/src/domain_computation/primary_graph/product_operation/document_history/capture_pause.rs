//! Receiver-local rendezvous between actual receipt capture and actual adoption.
use super::WorthQueryDocumentHistory;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct CapturePause {
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Default)]
struct State {
    reached: bool,
    released: bool,
}

#[doc(hidden)]
pub struct WorthQueryDocumentHistoryCapturePauseForTest {
    receiver: Arc<CapturePause>,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn pause_after_history_capture_for_test(
        &mut self,
    ) -> WorthQueryDocumentHistoryCapturePauseForTest {
        if let Some(previous) = &self.capture_pause {
            assert!(
                previous
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .released,
                "one capture pause may be armed"
            );
        }
        let receiver = Arc::new(CapturePause::default());
        self.capture_pause = Some(Arc::clone(&receiver));
        WorthQueryDocumentHistoryCapturePauseForTest { receiver }
    }
}

impl CapturePause {
    pub(super) fn pause(&self) {
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
            panic!("history capture handoff backstop expired");
        }
    }
}

impl WorthQueryDocumentHistoryCapturePauseForTest {
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
        self.receiver
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .released = true;
        self.receiver.changed.notify_all();
    }
}
impl Drop for WorthQueryDocumentHistoryCapturePauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
