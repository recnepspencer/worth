//! Callback over the selected exact-root value, with no scalar/carrier copy.
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope, RelationalRuntime},
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
};

pub(in crate::domain_computation::primary_graph) fn observe_field_value_borrowed<Output>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(Option<&'raw AspectValue>) -> Output,
) -> Result<Option<Output>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
{
    let Some(field) = locator.field_path().fields().first().cloned() else {
        return Ok(None);
    };
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    let mut action = Some(action);
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(None);
    };
    view.entity_record_with_projection_scope(execution, entity_id, scope, |record| {
        if record.kind_id() != kind || record.lifecycle() != RecordLifecycleState::Live {
            return None;
        }
        let action = action
            .take()
            .expect("single-entity projection invokes once");
        Some(action(
            record
                .observe_aspect_field(locator.aspect().aspect_key(), &field)
                .and_then(|observed| observed.value()),
        ))
    })
}

pub(in crate::domain_computation::primary_graph) fn observe_field_value_borrowed_admitted<
    Output,
>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(Option<&'raw AspectValue>) -> Output,
) -> Result<Option<Output>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
{
    observe_field_observation_borrowed_admitted(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        |observed| action(observed.value()),
    )
}

pub(in crate::domain_computation::primary_graph) fn observe_field_observation_borrowed_admitted<
    Output,
>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(
        worth_relational::facade::runtime::RelationalProjectedFieldObservation<'raw>,
    ) -> Output,
) -> Result<Option<Output>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
{
    let Some(field) = locator.field_path().fields().first().cloned() else {
        return Ok(None);
    };
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    let mut action = Some(action);
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(None);
    };
    // The operation reader already admitted this field into its carried work.
    view.entity_record_with_projection_scope_admitted(
        execution,
        entity_id,
        scope,
        |_| Ok::<_, std::convert::Infallible>(()),
        |record| {
            if record.kind_id() != kind || record.lifecycle() != RecordLifecycleState::Live {
                return None;
            }
            let action = action
                .take()
                .expect("single-entity projection invokes once");
            Some(action(
                record
                    .observe_aspect_field(locator.aspect().aspect_key(), &field)
                    .expect("the requested field belongs to its exact projection"),
            ))
        },
    )
    .map(|result| result.unwrap_or_else(|never| match never {}))
}

/// The rebase already admitted this native field probe. No present scalar is
/// copied merely to distinguish whole-aspect absence from present content.
pub(in crate::domain_computation::primary_graph) fn observe_aspect_absence(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
) -> Result<Option<bool>, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    observe_field_observation_borrowed_admitted(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        |raw| {
            use worth_relational::facade::runtime::RelationalProjectedFieldObservation as Raw;
            match raw {
                Raw::AspectAbsent => Some(true),
                Raw::FieldAbsent | Raw::Present(_) => Some(false),
                Raw::UnsupportedAspectShape => None,
            }
        },
    )
    .map(Option::flatten)
}
