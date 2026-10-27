//! Exact retirement equality shared by fact comparison and currentness.
pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    created_at: worth_relational::facade::identity::VersionId,
    deleted_at: worth_relational::facade::identity::VersionId,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    Ok(runtime
        .read_truth()
        .project_snapshot(snapshot)
        .map(|view| view.read_entity_retirement(execution, entity))
        .transpose()?
        .flatten()
        .is_some_and(|record| {
            record.kind_id() == kind
                && record.created_at_version() == created_at
                && record.deleted_at_version() == deleted_at
        }))
}
