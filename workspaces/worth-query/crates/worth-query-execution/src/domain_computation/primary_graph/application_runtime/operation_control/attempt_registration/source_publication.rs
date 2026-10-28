//! Literal closed control handles, each armed on its original runtime receiver.
use super::*;
mod receipt;
pub use receipt::{
    WorthQuerySourcePublicationHeadForTest, WorthQuerySourcePublicationImageForTest,
};
pub struct WorthQuerySourcePublicationValidationPauseForTest {
    slot: Arc<PendingAttemptPause>,
    latch: Arc<ExactAttemptPause>,
    observations: Arc<Mutex<Vec<WorthQuerySourcePublicationImageForTest>>>,
}
pub struct WorthQuerySourceFactComparisonPauseForTest {
    slot: Arc<PendingAttemptPause>,
    latch: Arc<ExactAttemptPause>,
}
impl WorthQueryApplicationAttemptOperationControl {
    pub(in crate::domain_computation::primary_graph) fn pause_after_source_publication_validation(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQuerySourcePublicationValidationPauseForTest {
        self.source_validation_observations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
        let latch = arm(&self.pending_source_validation, attempts);
        WorthQuerySourcePublicationValidationPauseForTest {
            slot: self.pending_source_validation.clone(),
            latch,
            observations: self.source_validation_observations.clone(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn source_validation_observed(&self) -> bool {
        self.pending_source_validation.armed.load(Ordering::Acquire)
    }
    pub(in crate::domain_computation::primary_graph) fn record_source_validation_image(
        &self,
        image: WorthQuerySourcePublicationImageForTest,
    ) {
        self.source_validation_observations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(image);
    }
    pub(in crate::domain_computation::primary_graph) fn after_source_publication_validation(&self) {
        reach(&self.pending_source_validation);
    }
    pub(in crate::domain_computation::primary_graph) fn pause_before_source_fact_comparison(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQuerySourceFactComparisonPauseForTest {
        let latch = arm(&self.pending_source_comparison, attempts);
        WorthQuerySourceFactComparisonPauseForTest {
            slot: self.pending_source_comparison.clone(),
            latch,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn before_source_fact_comparison(&self) {
        reach(&self.pending_source_comparison);
    }
}
fn wait(latch: &ExactAttemptPause, timeout: Duration) -> bool {
    latch.observe_arrivals(timeout)
}
fn release(slot: &PendingAttemptPause, latch: &Arc<ExactAttemptPause>) {
    clear(slot, latch);
    latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
    latch.changed.notify_all();
}
impl WorthQuerySourcePublicationValidationPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        wait(&self.latch, timeout)
    }
    pub fn release(&self) {
        release(&self.slot, &self.latch);
    }
    /// Diagnostic copies of actual Native coordinates/slots; grants no read or publication authority.
    pub fn observations(&self) -> Vec<WorthQuerySourcePublicationImageForTest> {
        self.observations
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}
impl Drop for WorthQuerySourcePublicationValidationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
impl WorthQuerySourceFactComparisonPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        wait(&self.latch, timeout)
    }
    pub fn release(&self) {
        release(&self.slot, &self.latch);
    }
}
impl Drop for WorthQuerySourceFactComparisonPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
