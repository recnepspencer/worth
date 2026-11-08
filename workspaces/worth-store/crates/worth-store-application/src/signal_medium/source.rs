//! One protected physical root supplies both Signal journal frontiers and bytes.
use super::*;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{DurableSignalOwnerIdentity, SignalDurableJournal};
use worth_store::physical_runtime::{OrderedJournalName, PhysicalRecordReader};
struct StoreSignalReadSource {
    owner: DurableSignalOwnerIdentity,
    serving: Arc<ServingPhysicalRuntime>,
    reader: Mutex<PhysicalRecordReader>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for StoreSignalReadSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StoreSignalReadSource")
    }
}
pub(super) fn capture(
    medium: &StoreSignalMedium,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Box<dyn SignalDurableReadSource>, D> {
    let funding = execution
        .reserve_retained_memory(std::mem::size_of::<StoreSignalReadSource>() as u64)
        .map_err(D::Request)?;
    let serving = medium.serving()?;
    let reader = serving
        .try_records_for_checkpoint(execution)
        .map_err(storage)?;
    let source = Box::new(StoreSignalReadSource {
        owner: medium.binding.owner(),
        serving,
        reader: Mutex::new(reader),
        _funding: funding,
    });
    match &medium.checkpoint {
        Some(image) => image.with_original_tail(source, execution),
        None => Ok(source),
    }
}
impl SignalDurableReadSource for StoreSignalReadSource {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    fn identity_contribution(
        &self,
    ) -> worth_signal::facade::durability::SignalRecordedIdentityContribution {
        worth_signal::facade::durability::SignalRecordedIdentityContribution::empty(self.owner)
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, D> {
        execution.consult().map_err(D::Request)?;
        let reader = self.reader.try_lock().map_err(|_| D::PublicationBusy)?;
        self.serving
            .ordered_journal(name(journal))
            .map_err(storage)?
            .head_position_from(&reader)
            .map_err(storage)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, D> {
        let reader = self.reader.try_lock().map_err(|_| D::PublicationBusy)?;
        let journal = self
            .serving
            .ordered_journal(name(journal))
            .map_err(storage)?;
        let mut stream = journal
            .open_payload_from(
                &reader,
                execution,
                position,
                worth_signal::facade::durability::SIGNAL_OWNER_FRAME_MAXIMUM_BYTES as u64,
            )
            .map_err(storage)?;
        let length: usize = stream
            .entry()
            .payload_bytes()
            .try_into()
            .map_err(|_| D::Unavailable)?;
        let mut bytes =
            ExecutionRequestArrayBuilder::allocate(*execution, length).map_err(D::Backing)?;
        let mut scratch = [0; 4096];
        loop {
            let count = stream.read_next(&mut scratch).map_err(storage)?;
            if count == 0 {
                break;
            }
            bytes
                .extend_from_slice(&scratch[..count])
                .map_err(D::Backing)?;
        }
        Ok(bytes.seal().map_err(D::Backing)?.into_immutable_bytes())
    }
}
pub(super) fn name(journal: SignalDurableJournal) -> OrderedJournalName {
    OrderedJournalName::new(journal.name()).expect("closed Signal journal namespace")
}
fn storage(cause: impl std::fmt::Debug) -> D {
    D::Storage(format!("{cause:?}"))
}
