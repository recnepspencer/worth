//! Pause only after an actual page has been assembled, before its final handoff guards.
use super::{receiver_pause::ReceiverPauseGuard, WorthQueryDocumentHistory};
use std::time::Duration;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
#[doc(hidden)]
pub struct WorthQueryHistoryPageAssemblyPauseForTest {
    guard: ReceiverPauseGuard,
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    #[doc(hidden)]
    pub fn pause_after_history_page_assembly_for_test(
        &mut self,
    ) -> WorthQueryHistoryPageAssemblyPauseForTest {
        WorthQueryHistoryPageAssemblyPauseForTest {
            guard: ReceiverPauseGuard::arm(&mut self.page_pause),
        }
    }
}
impl WorthQueryHistoryPageAssemblyPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.guard.wait_until_reached(timeout)
    }
    pub fn release(&self) {
        self.guard.release();
    }
}
