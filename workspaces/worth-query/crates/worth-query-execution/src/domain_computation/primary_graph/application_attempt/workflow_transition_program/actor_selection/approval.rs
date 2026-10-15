//! Recorded approval provenance; never reauthorizes its historical principal.
use super::*;
pub(super) fn approval_actor(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    transition: Option<WorkflowTransitionLocator>,
    proposal: Option<EntityId>,
    instance: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(Option<EntityId>, Option<EntityId>), WorthQueryApplicationAttemptDenial> {
    let Some(transition) = transition else {
        return Ok((None, None));
    };
    super::super::super::workflow_instance_observation::observe_retained_transition(
        execution, runtime, snapshot, layout, transition, facts,
    )?;
    let parent = super::super::super::workflow_instance_observation::original_parent(
        execution,
        runtime,
        snapshot,
        layout,
        transition.entity(),
        facts,
    )?;
    if parent != instance {
        return Err(affinity("original approval parent changed"));
    }
    let direction = super::super::super::WorthQueryApplicationAdjacencyDirection::Outgoing;
    let relations = super::super::super::observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.transition_approval_relation,
        transition.entity(),
        direction,
        2,
    )?
    .ok_or_else(|| affinity("workflow approval relation unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(affinity("workflow approval record is not singular"));
    };
    let entity = relation.to;
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: layout.transition_approval_relation,
        anchor: transition.entity(),
        direction,
        maximum_work_units: 2,
        relations,
    });
    let expected_proposal =
        proposal.ok_or_else(|| affinity("approval has no original proposal"))?;
    use super::super::super::super::workflow_instance_observation::instance_binding::{
        adjacency_with_kind, exact,
    };
    let proposals = adjacency_with_kind(
        execution,
        runtime,
        snapshot,
        layout.approval_proposal_relation,
        entity,
        2,
        "workflow approval proposal unavailable",
        facts,
    )?;
    if proposals.as_slice() != [expected_proposal] {
        return Err(affinity("approval belongs to another proposal occurrence"));
    }
    exact(execution, runtime, snapshot, entity, layout.approval.entity_kind,
        &layout.approval.protocol_version, worth_foundational::facade::AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION), facts)?;
    for (original, retained) in [
        (
            &layout.instance.identity,
            &layout.approval.instance_identity,
        ),
        (
            &layout.instance.definition_content_identity,
            &layout.approval.definition_content_identity,
        ),
    ] {
        let value = super::super::super::observe_field_value(
            runtime,
            execution,
            snapshot,
            instance,
            layout.instance.entity_kind,
            original,
        )?
        .ok_or_else(|| affinity("approval instance binding absent"))?;
        facts.push(WorthQueryApplicationObservedFact::Field {
            entity_id: instance,
            kind: layout.instance.entity_kind,
            locator: original.clone(),
            value: value.clone(),
        });
        exact(
            execution,
            runtime,
            snapshot,
            entity,
            layout.approval.entity_kind,
            retained,
            value,
            facts,
        )?;
    }
    WorkflowActorFields {
        partition: layout.approval.approver_partition.clone(),
        slot: layout.approval.approver_slot.clone(),
        generation: layout.approval.approver_generation.clone(),
    }
    .observe(
        runtime,
        execution,
        snapshot,
        entity,
        layout.approval.entity_kind,
        facts,
    )?
    .ok_or_else(|| affinity("current approval has no approver"))
    .map(|actor| (Some(actor), Some(entity)))
}
