//! The existing exact native lifecycle, retained as an output postcondition.
use super::*;
// The exact native retirement lookup and three lifecycle comparisons are
// prepaid before publication, including the commit's own deletion version.

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Option<RetirementMetadata>, CompanionPreflightStop> {
    let read = runtime.read_truth();
    let Some(view) = read.project_snapshot(snapshot) else {
        return Ok(None);
    };
    let Some(record) = view
        .read_entity_retirement(execution, entity)
        .map_err(CompanionPreflightStop::Execution)?
    else {
        return Ok(None);
    };
    Ok(Some(RetirementMetadata {
        kind: record.kind_id(),
        created_at: record.created_at_version(),
        deleted_at: record.deleted_at_version(),
    }))
}
