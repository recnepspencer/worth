//! Exact native captured contract DATA; installed authority is checked only at use.
use super::*;
pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    proposal_identity: &str,
    node: &str,
    entity: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorkflowCapturedInputContract, WorthQueryApplicationAttemptDenial> {
    let l = &layout.captured_input;
    let expected = super::super::super::captured_inputs::identity(proposal_identity, node);
    if required_text(
        execution,
        runtime,
        snapshot,
        entity,
        l.entity_kind,
        &l.identity,
        facts,
    )? != expected
    {
        return Err(denial(
            "captured target identity differs from its proposal and node",
        ));
    }
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
        resource: actual_resource,
        binding,
        input_type,
        input,
        key,
        source,
        preconditions,
    })
}
