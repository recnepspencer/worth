use super::{
    admission::BatchDeclaration,
    input::InputMode,
    meter::KernelFailure,
    port::{BatchOutcome, BatchStop},
    PreparedBatchResources,
};
use crate::authority::{ExecutionMemoryReservation, ResourceReservation};

/// Fields release in this order: inputs, entry, serial hold, taken hold, prepared.
/// In particular an inline zero-byte entry still needs its lineage nodes:
/// releasing the taken hold first can remove those nodes before entry release.
/// On takeover refusal the entry closes before cleanup and the taken hold.
/// On unwind the entry releases after `limits`, which was declared after custody.
pub(super) struct RunCustody<I: InputMode> {
    pub(super) inputs: Option<I>,
    pub(super) reservation: Option<ResourceReservation>,
    pub(super) serial_reservation: Option<ExecutionMemoryReservation>,
    pub(super) taken: Option<ExecutionMemoryReservation>,
    pub(super) prepared: Option<PreparedBatchResources>,
}

impl<I: InputMode> RunCustody<I> {
    pub(super) fn new(
        inputs: I,
        taken: Option<ExecutionMemoryReservation>,
        prepared: Option<PreparedBatchResources>,
    ) -> Self {
        Self {
            inputs: Some(inputs),
            reservation: None,
            serial_reservation: None,
            taken,
            prepared,
        }
    }
    pub(super) fn finish<R, E>(
        &mut self,
        mut outcome: BatchOutcome<R, E>,
        batch: &BatchDeclaration,
    ) -> BatchOutcome<R, E> {
        if let Some(index) = self.inputs.as_mut().and_then(InputMode::discard) {
            // No input was dispatched on this path, so there is no prefix.
            // Cleanup still reports the least destructor failure identity.
            outcome.stop = Some(BatchStop::Failure {
                identity: batch.identities()[index],
                cause: KernelFailure::Panic,
            });
        }
        // Release normal-run guards before local meter contexts disappear.
        // Prepared resources remain last.
        drop(self.inputs.take());
        drop(self.reservation.take());
        drop(self.serial_reservation.take());
        drop(self.taken.take());
        outcome
    }
}
impl<I: InputMode> Drop for RunCustody<I> {
    fn drop(&mut self) {
        // On an unrelated unwind preserve that original panic, while destroying
        // every remaining input under custody before Rust releases the fields.
        if let Some(inputs) = self.inputs.as_mut() {
            inputs.discard();
        }
    }
}
