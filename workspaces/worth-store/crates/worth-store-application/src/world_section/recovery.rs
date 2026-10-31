use super::ApplicationWorldMedium;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldDurableJournal;
use worth_runtime_world::facade::{
    WorldDurableMediumDenial as Denial, WorldDurableRecoverySource, WorldRecoveredJournalRecord,
};
use worth_store::physical_runtime::OrderedJournalName;
pub(super) fn read(
    medium: &ApplicationWorldMedium,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldDurableRecoverySource, Denial> {
    Ok(WorldDurableRecoverySource::from_owner_records(
        read_stream(medium, WorldDurableJournal::Branches, execution)?,
        read_stream(medium, WorldDurableJournal::Occurrences, execution)?,
        read_stream(medium, WorldDurableJournal::Custody, execution)?,
    ))
}
fn read_stream(
    medium: &ApplicationWorldMedium,
    stream: WorldDurableJournal,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Vec<WorldRecoveredJournalRecord>, Denial> {
    execution.consult().map_err(Denial::Request)?;
    let journal = medium
        .serving
        .ordered_journal(OrderedJournalName::new(stream.name()).expect("closed World journal"))
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
    let head = journal
        .head()
        .map_err(|cause| Denial::Storage(format!("{cause:?}")))?
        .map_or(0, |head| head.position());
    let mut records = Vec::new();
    for position in 1..=head {
        execution.consult().map_err(Denial::Request)?;
        let payload = journal
            .read(position, 64 << 10)
            .map_err(|cause| Denial::Storage(format!("{cause:?}")))?;
        let entry = payload.entry();
        records.push(WorldRecoveredJournalRecord::from_owner_record(
            entry.position(),
            entry.payload_sha256(),
            payload.into_bytes(),
        ));
    }
    Ok(records)
}
