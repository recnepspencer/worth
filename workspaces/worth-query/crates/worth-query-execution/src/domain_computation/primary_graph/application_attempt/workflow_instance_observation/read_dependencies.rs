//! Bounded dependency identities, including anchors of currently absent workflow edges.
use super::read_fields::ReadFields;
use super::*;
use crate::domain_computation::primary_graph::application_discovery::WorthQueryWorkflowReadDenial as Denial;
pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    lineage: EntityId,
    history: &ReadOnlyWorkflowHistory,
) -> Result<worth_execution::ExecutionArray<EntityId>, Denial> {
    let capacity = history
        .transition_count()
        .checked_mul(5)
        .and_then(|v| v.checked_add(3))
        .ok_or(Denial::Corrupt)?;
    let mut result =
        worth_execution::ExecutionArrayBuilder::allocate(capacity, execution.allocation_policy())
            .map_err(Denial::Allocation)?;
    for id in [instance, lineage] {
        result.push(id).map_err(Denial::Allocation)?;
    }
    let fields = ReadFields::new(runtime, execution, snapshot)?;
    let definition = fields
        .one(layout.instance_definition_relation, instance)?
        .ok_or(Denial::Corrupt)?;
    result.push(definition).map_err(Denial::Allocation)?;
    for transition in history.transitions() {
        // The transition anchor also covers absent proposal/approval/evidence edges.
        result.push(transition.entity).map_err(Denial::Allocation)?;
        result
            .push(transition.settlement.node())
            .map_err(Denial::Allocation)?;
        for relation in [
            layout.transition_proposal_relation,
            layout.transition_approval_relation,
            layout.transition_assessment_evidence_relation,
        ] {
            // An absent edge consumes its fixed slot with its actual anchor, not a fabricated target.
            let dependency = fields
                .one(relation, transition.entity)?
                .unwrap_or(transition.entity);
            result.push(dependency).map_err(Denial::Allocation)?;
        }
    }
    result.seal().map_err(Denial::Allocation)
}
