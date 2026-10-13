//! Scoped receiver-owned residue refusal for readmission's cleanup proof.
use super::WorthQueryApplicationPreviewSession;
pub(super) struct PreviewDiscardFaultForTest(std::rc::Rc<std::cell::Cell<bool>>);
impl Drop for PreviewDiscardFaultForTest {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
impl WorthQueryApplicationPreviewSession {
    pub(super) fn reject_discard_for_test(&mut self) -> PreviewDiscardFaultForTest {
        self.discard_fault.set(true);
        PreviewDiscardFaultForTest(std::rc::Rc::clone(&self.discard_fault))
    }
}
