//! Receiver-local rendezvous between actual receipt capture and actual adoption.
use super::{receiver_pause::ReceiverPauseGuard, WorthQueryDocumentHistory};
use std::time::Duration;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
#[doc(hidden)]
pub struct WorthQueryDocumentHistoryCapturePauseForTest {
    guard: ReceiverPauseGuard,
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn pause_after_history_capture_for_test(
        &mut self,
    ) -> WorthQueryDocumentHistoryCapturePauseForTest {
        WorthQueryDocumentHistoryCapturePauseForTest {
            guard: ReceiverPauseGuard::arm(&mut self.capture_pause),
        }
    }
}
impl WorthQueryDocumentHistoryCapturePauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.guard.wait_until_reached(timeout)
    }
    pub fn release(&self) {
        self.guard.release();
    }
}
