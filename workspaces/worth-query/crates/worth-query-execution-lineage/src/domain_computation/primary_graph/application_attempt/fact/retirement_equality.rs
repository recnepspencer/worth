//! Retirement equality: exact for fact comparison and currentness, and
//! generation-only for a correction's paired target comparison.
pub fn remains_equal(
    view: Option<&worth_relational::facade::runtime::VisibilityProjectionView>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: worth_relational::facade::identity::EntityId,
    kind: worth_relational::facade::identity::KindId,
    created_at: worth_relational::facade::identity::VersionId,
    deleted_at: worth_relational::facade::identity::VersionId,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    Ok(view
        .map(|view| view.read_entity_retirement(execution, entity))
        .transpose()?
        .flatten()
        .is_some_and(|record| {
            record.kind_id() == kind
                && record.created_at_version() == created_at
                && record.deleted_at_version() == deleted_at
        }))
}

/// Whether the exact generation `entity` is still retired at `view`: the
/// slot holds it and its lifecycle is retained-deleted. A correction's
/// paired comparison reads only this at its target (public #445), because
/// the original side is already pinned exactly. `remains_equal` stays exact
/// on `deleted_at` for postcommit currentness, where any later commit that
/// revived and retired the generation again is a change since the seal.
pub fn still_retired(
    view: Option<&worth_relational::facade::runtime::VisibilityProjectionView>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity: worth_relational::facade::identity::EntityId,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    Ok(view
        .map(|view| view.read_entity_retirement(execution, entity))
        .transpose()?
        .flatten()
        .is_some())
}
