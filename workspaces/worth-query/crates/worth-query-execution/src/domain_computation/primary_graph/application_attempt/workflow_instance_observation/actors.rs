//! Canonical instance role applicability at the selected native basis.
use super::*;
/// Already validated recorded identities; data only, never authority.
pub(in crate::domain_computation::primary_graph) struct WorkflowInstanceRoles {
    pub(in crate::domain_computation::primary_graph) start: EntityId,
    pub(in crate::domain_computation::primary_graph) end:
        Option<worth_foundational::facade::WorkflowEndingActorProvenance>,
}
pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowInstanceRoles, WorthQueryApplicationAttemptDenial> {
    let kind = layout.instance.entity_kind;
    let applicable = crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability;
    let start_actor = layout
        .instance
        .start_advancer
        .observe(runtime, execution, snapshot, entity, kind, facts)?;
    applicable(start_actor, true)?;
    let end_actor = layout
        .instance
        .ending_actor
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
    let ended = match WorkflowInstanceState::from_persisted_tag(tag) {
        Some(WorkflowInstanceState::Ready) => false,
        Some(
            WorkflowInstanceState::Completed
            | WorkflowInstanceState::Cancelled
            | WorkflowInstanceState::Terminated
            | WorkflowInstanceState::Migrated,
        ) => true,
        None => return Err(denial("workflow instance state is unsupported")),
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: layout.instance.state.clone(),
        value: AspectValue::UInt64(tag),
    });
    if end_actor.is_some() != ended {
        return Err(crate::domain_computation::primary_graph::workflow::actor::record::actor_record_corruption());
    }
    let Some(start) = start_actor else {
        return Err(denial("current workflow instance has no starting advancer"));
    };
    Ok(WorkflowInstanceRoles {
        start,
        end: end_actor,
    })
}
