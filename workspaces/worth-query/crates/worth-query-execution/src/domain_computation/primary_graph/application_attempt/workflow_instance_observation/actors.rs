//! Canonical instance role applicability at the selected native basis.
use super::*;
pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let kind = layout.instance.entity_kind;
    let applicable = crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability;
    let start_actor = layout
        .instance
        .start_advancer
        .observe(runtime, execution, snapshot, entity, kind, facts)?;
    if applicable(start_actor, true).is_err() {
        return Err(denial("current workflow instance has no starting advancer"));
    }
    let end_actor = layout
        .instance
        .end_advancer
        .observe(runtime, execution, snapshot, entity, kind, facts)?;
    let state = observe_field_value(
        runtime,
        execution,
        snapshot,
        entity,
        kind,
        &layout.instance.state,
    )?;
    let Some(AspectValue::UInt64(tag)) = state else {
        return Err(denial("workflow instance state is malformed"));
    };
    let ended = if tag == WorkflowInstanceState::Ready.persisted_tag() {
        false
    } else if [
        WorkflowInstanceState::Completed,
        WorkflowInstanceState::Cancelled,
        WorkflowInstanceState::Migrated,
    ]
    .iter()
    .any(|state| state.persisted_tag() == tag)
    {
        true
    } else {
        return Err(denial("workflow instance state is unsupported"));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: layout.instance.state.clone(),
        value: AspectValue::UInt64(tag),
    });
    if applicable(end_actor, ended).is_err() {
        return Err(denial(
            "workflow instance ending actor applicability is corrupt",
        ));
    }
    Ok(())
}
