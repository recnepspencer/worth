//! The existing exact native lifecycle, retained as an output postcondition.
use super::*;
// The exact native retirement lookup and three lifecycle comparisons are
// prepaid before publication, including the commit's own deletion version.
pub(super) const SEAL_WORK: u64 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetirementMetadata {
    pub(super) kind: KindId,
    pub(super) created_at: VersionId,
    pub(super) deleted_at: VersionId,
}

pub(super) fn native_metadata(
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
) -> Option<RetirementMetadata> {
    let read = runtime.read_truth();
    let view = read.project_snapshot(snapshot)?;
    let record = view.entity_retirement(entity)?;
    Some(RetirementMetadata {
        kind: record.kind_id(),
        created_at: record.created_at_version(),
        deleted_at: record.deleted_at_version(),
    })
}
