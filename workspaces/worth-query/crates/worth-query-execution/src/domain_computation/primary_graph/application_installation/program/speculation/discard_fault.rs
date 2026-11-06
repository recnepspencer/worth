//! Scoped receiver-owned residue refusal for readmission's cleanup proof.
use super::WorthQueryApplicationPreviewSession;
pub struct PreviewDiscardFaultForTest(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl Drop for PreviewDiscardFaultForTest {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
impl WorthQueryApplicationPreviewSession {
    pub fn reject_discard_for_test(&mut self) -> PreviewDiscardFaultForTest {
        self.discard_fault
            .store(true, std::sync::atomic::Ordering::SeqCst);
        PreviewDiscardFaultForTest(std::sync::Arc::clone(&self.discard_fault))
    }
}
