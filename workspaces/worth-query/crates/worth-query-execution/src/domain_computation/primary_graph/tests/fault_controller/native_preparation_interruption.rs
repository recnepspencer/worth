//! Receiver-owned unit-test control at the existing native preparation hook.
use super::*;
pub(in crate::domain_computation::primary_graph) struct NativePreparationInterruptionForTest<'a> {
    owner: &'a PrimaryGraphFaultController,
    before: usize,
}
impl PrimaryGraphFaultController {
    pub(in crate::domain_computation::primary_graph) fn interrupt_next_native_preparation_for_test(
        &self,
        source: WorthQueryCancellationSource,
        deadline: Option<std::time::Instant>,
    ) -> NativePreparationInterruptionForTest<'_> {
        *self.native_preparation_cancellation.lock().unwrap() = Some((source, deadline));
        NativePreparationInterruptionForTest {
            owner: self,
            before: self
                .native_preparation_interruptions
                .load(Ordering::Acquire),
        }
    }
}
impl NativePreparationInterruptionForTest<'_> {
    pub(in crate::domain_computation::primary_graph) fn reached_for_test(&self) -> bool {
        self.owner
            .native_preparation_interruptions
            .load(Ordering::Acquire)
            == self.before + 1
    }
}
impl Drop for NativePreparationInterruptionForTest<'_> {
    fn drop(&mut self) {
        self.owner
            .native_preparation_cancellation
            .lock()
            .unwrap()
            .take();
    }
}
