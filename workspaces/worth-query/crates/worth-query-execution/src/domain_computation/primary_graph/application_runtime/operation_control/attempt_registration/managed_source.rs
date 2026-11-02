//! Park only a real checked target after the runtime loan and before Native pinning.
use super::*;

pub struct WorthQueryManagedSourceRevalidationPauseForTest {
    slot: Arc<PendingAttemptPause>,
    latch: Arc<ExactAttemptPause>,
}
impl WorthQueryApplicationAttemptOperationControl {
    pub(in crate::domain_computation::primary_graph) fn pause_after_managed_source_revalidation(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQueryManagedSourceRevalidationPauseForTest {
        WorthQueryManagedSourceRevalidationPauseForTest {
            latch: arm(&self.pending_managed_revalidation, attempts),
            slot: self.pending_managed_revalidation.clone(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn after_managed_source_revalidation(&self) {
        reach(&self.pending_managed_revalidation);
    }
}
impl WorthQueryManagedSourceRevalidationPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.latch.observe_arrivals(timeout)
    }
    pub fn release(&self) {
        clear(&self.slot, &self.latch);
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}
impl Drop for WorthQueryManagedSourceRevalidationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
