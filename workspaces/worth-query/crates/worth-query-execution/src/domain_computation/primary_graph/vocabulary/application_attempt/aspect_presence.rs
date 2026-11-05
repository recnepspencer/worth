//! Moved whole from `domain_computation::primary_graph::application_attempt::observation::aspect_presence`: the items below and every impl they own.
use worth_foundational::facade::AspectKey;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::storage::RecordLifecycleState;

pub fn observe_aspect_presence(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    aspect: &AspectKey,
) -> Result<Option<bool>, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(None);
    };
    view.entity_record_with_projection_scope(
        execution,
        entity,
        ProjectionAspectScope::whole_aspects([aspect.clone()]),
        |record| {
            (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(
                || {
                    record.struct_aspect_value(aspect).is_some()
                        || record.aspect_value(aspect).is_some()
                },
            )
        },
    )
}
