//! Pause after authentic context evidence, before final current-source guards.
use super::{receiver_pause::ReceiverPauseGuard, WorthQueryDocumentHistory};
use std::time::Duration;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
#[doc(hidden)]
pub struct WorthQueryHistoryContextRefreshPauseForTest {
    guard: ReceiverPauseGuard,
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn pause_before_history_context_refresh_for_test(
        &mut self,
    ) -> WorthQueryHistoryContextRefreshPauseForTest {
        WorthQueryHistoryContextRefreshPauseForTest {
            guard: ReceiverPauseGuard::arm(&mut self.context_pause),
        }
    }
}
impl WorthQueryHistoryContextRefreshPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.guard.wait_until_reached(timeout)
    }
    pub fn release(&self) {
        self.guard.release();
    }
}
