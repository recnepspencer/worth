//! Callback over the selected exact-root value, with no scalar/carrier copy.
//! The projection visit is compiled once. Each caller only slots its output.
use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        ProjectionAspectRequirement, ProjectionAspectScope, RelationalProjectedFieldObservation,
        RelationalRuntime,
    },
    snapshots::SnapshotHandle,
    storage::RecordLifecycleState,
};

type RelationalDenial = worth_relational::facade::transactions::RelationalExecutionDenialCause;

pub(in crate::domain_computation::primary_graph) fn observe_field_value_borrowed<Output>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(Option<&'raw AspectValue>) -> Output,
) -> Result<Option<Output>, RelationalDenial> {
    let mut output = None;
    let mut action = Some(action);
    let invoked = observe_field_value_borrowed_core(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        &mut |value| assign_borrowed(&mut action, &mut output, value),
    )?;
    Ok(invoked_output(invoked, output))
}

pub(in crate::domain_computation::primary_graph) fn observe_field_value_borrowed_admitted<Output>(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: impl for<'raw> FnOnce(Option<&'raw AspectValue>) -> Output,
) -> Result<Option<Output>, RelationalDenial> {
    let mut output = None;
    let mut action = Some(action);
    let invoked = observe_field_value_borrowed_admitted_core(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        &mut |value| assign_borrowed(&mut action, &mut output, value),
    )?;
    Ok(invoked_output(invoked, output))
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
    action: impl for<'raw> FnOnce(RelationalProjectedFieldObservation<'raw>) -> Output,
) -> Result<Option<Output>, RelationalDenial> {
    let mut output = None;
    let mut action = Some(action);
    let invoked = observe_field_observation_borrowed_admitted_core(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        &mut |value| assign_observation(&mut action, &mut output, value),
    )?;
    Ok(invoked_output(invoked, output))
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
) -> Result<Option<bool>, RelationalDenial> {
    let mut absence = None;
    let invoked = observe_field_observation_borrowed_admitted_core(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        &mut |raw| {
            absence = Some(match raw {
                RelationalProjectedFieldObservation::AspectAbsent => Some(true),
                RelationalProjectedFieldObservation::FieldAbsent
                | RelationalProjectedFieldObservation::Present(_) => Some(false),
                RelationalProjectedFieldObservation::UnsupportedAspectShape => None,
            });
        },
    )?;
    Ok(invoked
        .then(|| absence.expect("single-entity projection invokes once"))
        .flatten())
}

#[inline(never)]
fn observe_field_value_borrowed_core(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: &mut dyn for<'raw> FnMut(Option<&'raw AspectValue>),
) -> Result<bool, RelationalDenial> {
    let Some((field, scope)) = projected_field_scope(locator) else {
        return Ok(false);
    };
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(false);
    };
    view.entity_record_with_projection_scope(execution, entity_id, scope, |record| {
        if record.kind_id() != kind || record.lifecycle() != RecordLifecycleState::Live {
            return None;
        }
        action(
            record
                .observe_aspect_field(locator.aspect().aspect_key(), &field)
                .and_then(|observed| observed.value()),
        );
        Some(())
    })
    .map(|invoked| invoked.is_some())
}

#[inline(never)]
fn observe_field_value_borrowed_admitted_core(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: &mut dyn for<'raw> FnMut(Option<&'raw AspectValue>),
) -> Result<bool, RelationalDenial> {
    observe_field_observation_borrowed_admitted_core(
        runtime,
        execution,
        snapshot,
        entity_id,
        kind,
        locator,
        &mut |observed| action(observed.value()),
    )
}

#[inline(never)]
fn observe_field_observation_borrowed_admitted_core(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    entity_id: EntityId,
    kind: KindId,
    locator: &AspectFieldLocator,
    action: &mut dyn for<'raw> FnMut(RelationalProjectedFieldObservation<'raw>),
) -> Result<bool, RelationalDenial> {
    let Some((field, scope)) = projected_field_scope(locator) else {
        return Ok(false);
    };
    let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
        return Ok(false);
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
            action(
                record
                    .observe_aspect_field(locator.aspect().aspect_key(), &field)
                    .expect("the requested field belongs to its exact projection"),
            );
            Some(())
        },
    )
    .map(|result| result.unwrap_or_else(|never| match never {}).is_some())
}

fn projected_field_scope(locator: &AspectFieldLocator) -> Option<(FieldKey, ProjectionAspectScope)> {
    let field = locator.field_path().fields().first().cloned()?;
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    Some((field, scope))
}

fn assign_borrowed<'raw, Output>(
    action: &mut Option<impl for<'a> FnOnce(Option<&'a AspectValue>) -> Output>,
    output: &mut Option<Output>,
    value: Option<&'raw AspectValue>,
) {
    let action = action
        .take()
        .expect("single-entity projection invokes once");
    *output = Some(action(value));
}

fn assign_observation<'raw, Output>(
    action: &mut Option<impl for<'a> FnOnce(RelationalProjectedFieldObservation<'a>) -> Output>,
    output: &mut Option<Output>,
    value: RelationalProjectedFieldObservation<'raw>,
) {
    let action = action
        .take()
        .expect("single-entity projection invokes once");
    *output = Some(action(value));
}

fn invoked_output<Output>(invoked: bool, output: Option<Output>) -> Option<Output> {
    if invoked {
        Some(output.expect("single-entity projection invokes once"))
    } else {
        None
    }
}
