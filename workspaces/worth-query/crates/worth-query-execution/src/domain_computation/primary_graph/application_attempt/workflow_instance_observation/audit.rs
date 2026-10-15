//! Value handoff from the canonical actor reader, used only after the independent audit gate.
use super::*;
use crate::domain_computation::primary_graph::application_discovery::{
    WorthQueryWorkflowAudit, WorthQueryWorkflowReadDenial as Denial,
    WorthQueryWorkflowRecordedActor as Actor, WorthQueryWorkflowTransitionActors,
};
use worth_execution::ExecutionArrayBuilder;

pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    history: &ReadOnlyWorkflowHistory,
) -> Result<WorthQueryWorkflowAudit, Denial> {
    if !history.matches_target(instance, snapshot) {
        return Err(Denial::PolicyMismatch);
    }
    // Allocate the complete bounded role collection before reading any identity.
    let mut result =
        ExecutionArrayBuilder::allocate(history.transition_count(), execution.allocation_policy())
            .map_err(Denial::Allocation)?;
    let mut facts = Vec::new();
    let roles = observe_instance_roles(execution, runtime, snapshot, layout, instance, &mut facts)
        .map_err(Denial::Execution)?;
    for transition in history.transitions() {
        facts.clear();
        let roles = observe_transition_roles(
            execution,
            runtime,
            snapshot,
            layout,
            transition.entity,
            transition.settlement.operation_receipt_identity().is_some(),
            &mut facts,
        )
        .map_err(Denial::Execution)?;
        result
            .push(WorthQueryWorkflowTransitionActors::observed(
                Actor::observed(roles.advancer),
                roles.executor.map(Actor::observed),
            ))
            .map_err(Denial::Allocation)?;
    }
    let transitions = result.seal().map_err(Denial::Allocation)?;
    Ok(WorthQueryWorkflowAudit::Visible {
        start: Actor::observed(roles.start),
        end: roles.end.map(Actor::observed),
        transitions,
    })
}
