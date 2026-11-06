//! Receiver-owned unit-test control at the existing native preparation hook.
use crate::domain_computation::primary_graph::tests::fault_controller::*;
pub struct NativePreparationInterruptionForTest<'a> {
    pub owner: &'a PrimaryGraphFaultController,
    pub before: usize,
}
impl PrimaryGraphFaultController {
    pub fn interrupt_next_native_preparation_for_test(
        &self,
        source: WorthQueryCancellationSource,
        deadline: Option<(worth_execution::MonotonicTimeController, std::time::Instant)>,
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
    pub fn reached_for_test(&self) -> bool {
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
