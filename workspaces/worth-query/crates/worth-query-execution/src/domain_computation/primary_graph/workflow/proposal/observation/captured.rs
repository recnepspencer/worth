//! Native profile decoding and exact indexed target selection; no input recapture.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::observe_indexed_entity_selection,
    workflow::{captured_input::WorkflowCapturedInputContract, schema::version::*},
};

pub(super) fn profile(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<([u8; 32], u64)>, WorthQueryApplicationAttemptDenial> {
    let l = &layout.proposal;
    let protocol = required_u64(
        execution,
        runtime,
        snapshot,
        proposal,
        l.entity_kind,
        &l.protocol_version,
        facts,
    )?;
    let captured = optional_identity(
        execution,
        runtime,
        snapshot,
        proposal,
        l.entity_kind,
        &l.captured_identity,
        facts,
    )?;
    let count = observe_field_value(
        runtime,
        execution,
        snapshot,
        proposal,
        l.entity_kind,
        &l.captured_count,
    )?;
    match (protocol, captured, count) {
        (WORKFLOW_FACT_PROTOCOL_VERSION, None, None) => {
            facts.push(WorthQueryApplicationObservedFact::AbsentField {
                entity_id: proposal,
                kind: l.entity_kind,
                locator: l.captured_count.clone(),
            });
            Ok(None)
        }
        (
            WORKFLOW_CAPTURED_PROPOSAL_PROTOCOL_VERSION,
            Some(identity),
            Some(AspectValue::UInt64(count)),
        ) if count > 0 => {
            facts.push(WorthQueryApplicationObservedFact::Field {
                entity_id: proposal,
                kind: l.entity_kind,
                locator: l.captured_count.clone(),
                value: AspectValue::UInt64(count),
            });
            Ok(Some((identity, count)))
        }
        _ => Err(denial("workflow captured proposal profile is invalid")),
    }
}

pub(in crate::domain_computation::primary_graph) fn target(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    proposal_identity: &str,
    node: &str,
    resource: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowCapturedInputContract, WorthQueryApplicationAttemptDenial> {
    let l = &layout.captured_input;
    let identity = super::super::captured_inputs::identity(proposal_identity, node);
    let selection = observe_indexed_entity_selection(
        runtime,
        execution,
        snapshot,
        l.identity_index_id,
        l.entity_kind,
        l.identity.clone(),
        AspectValue::String(InternedString::Raw(identity)),
        2,
    )
    .map_err(|cause| match cause {
        crate::domain_computation::primary_graph::application_attempt::WorthQueryIndexedSelectionRefusal::Execution(cause) =>
            WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "captured workflow target"),
        _ => denial("captured workflow target selection is unavailable"),
    })?;
    let WorthQueryApplicationObservedFact::IndexedEntitySelection { candidates, .. } = &selection
    else {
        unreachable!("native indexed selection")
    };
    let [entity] = candidates.as_slice() else {
        return Err(denial("captured workflow input is not unique"));
    };
    let entity = *entity;
    facts.push(selection);
    let incoming = one_reference(
        execution,
        runtime,
        snapshot,
        l.membership,
        entity,
        WorthQueryApplicationAdjacencyDirection::Incoming,
        facts,
    )?;
    let actual_resource = one_reference(
        execution,
        runtime,
        snapshot,
        l.resource,
        entity,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        facts,
    )?;
    if incoming != proposal
        || actual_resource != resource
        || required_text(
            execution,
            runtime,
            snapshot,
            entity,
            l.entity_kind,
            &l.node,
            facts,
        )? != node
    {
        return Err(denial("captured workflow target correspondence changed"));
    }
    let binding = required_text(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.binding,
        facts,
    )?;
    let input_type = required_text(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.input_type,
        facts,
    )?;
    let mut identity = |field: &worth_foundational::facade::AspectFieldLocator| {
        let text = required_text(
            execution,
            runtime,
            snapshot,
            entity,
            l.entity_kind,
            field,
            facts,
        )?;
        decode_identity(&text).ok_or_else(|| denial("captured request identity is malformed"))
    };
    let input = identity(&l.input)?;
    let key = identity(&l.key)?;
    let source_kind = required_u64(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.source_kind,
        facts,
    )?;
    let source = optional_identity(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.source,
        facts,
    )?;
    let partition = optional_identity(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.source_partition,
        facts,
    )?;
    let source = match (source_kind, source, partition) {
        (0, None, None) => None,
        (kind @ (1 | 2), Some(identity), Some(partition)) => Some((kind, identity, partition)),
        _ => return Err(denial("captured source shape is invalid")),
    };
    let preconditions = optional_identity(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.preconditions,
        facts,
    )?;
    Ok(WorkflowCapturedInputContract {
        resource,
        binding,
        input_type,
        input,
        key,
        source,
        preconditions,
    })
}

pub(super) fn one_reference(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    relation: worth_relational::facade::identity::KindId,
    anchor: EntityId,
    direction: WorthQueryApplicationAdjacencyDirection,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let rows = observe_adjacency(runtime, execution, snapshot, relation, anchor, direction, 2)?
        .ok_or_else(|| denial("captured input reference is unavailable"))?;
    let [one] = rows.as_slice() else {
        return Err(denial("captured input reference is not unique"));
    };
    let target = if direction == WorthQueryApplicationAdjacencyDirection::Outgoing {
        one.to
    } else {
        one.from
    };
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: relation,
        anchor,
        direction,
        maximum_work_units: 2,
        relations: rows,
    });
    Ok(target)
}
