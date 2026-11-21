//! Recorded transition identities, decoded and checked once at the supplied basis.
use super::*;
pub struct WorkflowTransitionRoles
{
    pub advancer: EntityId,
    pub executor:
        Option<EntityId>,
}
pub fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    has_operation_receipt: bool,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowTransitionRoles, WorthQueryApplicationAttemptDenial> {
    let kind = layout.transition.entity_kind;
    let advancer =
        crate::domain_computation::primary_graph::workflow::actor::record::observe_actor_fields(
            &layout.transition.advancer,
            runtime,
            execution,
            snapshot,
            transition,
            kind,
            facts,
        )?;
    let executor =
        crate::domain_computation::primary_graph::workflow::actor::record::observe_actor_fields(
            &layout.transition.executor,
            runtime,
            execution,
            snapshot,
            transition,
            kind,
            facts,
        )?;
    let applicable = crate::domain_computation::primary_graph::workflow::actor::record::validate_actor_applicability;
    applicable(advancer, true)?;
    applicable(executor, has_operation_receipt)?;
    let Some(advancer) = advancer else {
        return Err(denial("current transition has no advancer"));
    };
    Ok(WorkflowTransitionRoles { advancer, executor })
}
