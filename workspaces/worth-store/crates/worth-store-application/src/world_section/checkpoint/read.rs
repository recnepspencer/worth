//! World receives only the SAME original Home cut's verified owner bytes.
use super::*;
use worth_foundational::facade::WorldDurableJournal;
use worth_runtime_world::facade::{WorldCheckpointJournalRecord, WorldCheckpointReadView};
use worth_store::physical_runtime::OrderedJournalName;

pub(super) fn view(
    source: &ApplicationWorldCheckpointSource,
    request: &ExecutionRequest<'_, '_>,
) -> Result<WorldCheckpointReadView, Denial> {
    let selected = source
        .selected
        .selected(&source.serving, request)
        .map_err(denial)?;
    let selected = selected.as_ref().map(|manifest| manifest.world_region());
    let mut heads = [0; 3];
    for (slot, journal) in [
        WorldDurableJournal::Branches,
        WorldDurableJournal::Custody,
        WorldDurableJournal::Occurrences,
    ]
    .into_iter()
    .enumerate()
    {
        heads[slot] = source
            .selected
            .journal_head(&source.serving, name(journal), request)
            .map_err(denial)?;
    }
    WorldCheckpointReadView::from_protected_home_source(selected, heads)
}
pub(super) fn journal(
    source: &ApplicationWorldCheckpointSource,
    request: &ExecutionRequest<'_, '_>,
    journal: WorldDurableJournal,
    position: u64,
) -> Result<WorldCheckpointJournalRecord, Denial> {
    let (entry, bytes) = source
        .selected
        .journal_payload(&source.serving, name(journal), position, request)
        .map_err(denial)?;
    Ok(WorldCheckpointJournalRecord::from_protected_world_journal(
        entry.position(),
        entry.payload_sha256(),
        bytes,
    ))
}
fn name(journal: WorldDurableJournal) -> OrderedJournalName {
    OrderedJournalName::new(journal.name()).expect("closed World journal")
}
pub(super) fn denial(cause: crate::ApplicationHomeCheckpointDenial) -> Denial {
    match cause {
        crate::ApplicationHomeCheckpointDenial::Execution(cause) => Denial::Request(cause),
        crate::ApplicationHomeCheckpointDenial::Payload(cause) => Denial::Backing(cause),
        crate::ApplicationHomeCheckpointDenial::ReadSourceBusy => Denial::PublicationBusy,
        crate::ApplicationHomeCheckpointDenial::CheckpointReadProtection(
            worth_store::physical_runtime::PhysicalCheckpointReadProtectionDenial::Busy,
        ) => Denial::PublicationBusy,
        cause => Denial::Storage(format!("{cause:?}")),
    }
}
