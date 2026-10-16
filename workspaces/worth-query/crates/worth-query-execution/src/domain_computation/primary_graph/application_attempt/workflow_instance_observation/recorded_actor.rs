//! Original immutable occurrence membership, independent of command eligibility.
use super::*;
use crate::domain_computation::primary_graph::workflow::instance::WorkflowTransitionLocator;

pub(in crate::domain_computation::primary_graph::application_attempt) fn original_parent(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    incoming(
        execution,
        runtime,
        snapshot,
        layout.instance_transition_relation,
        transition,
        facts,
    )
}
pub(in crate::domain_computation::primary_graph::application_attempt) fn original_transition(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    proposal: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(WorkflowTransitionLocator, String), WorthQueryApplicationAttemptDenial> {
    let entity = incoming(
        execution,
        runtime,
        snapshot,
        layout.transition_proposal_relation,
        proposal,
        facts,
    )?;
    let settlement = settlement::observe_settled_transition(
        execution, runtime, snapshot, layout, entity, facts,
    )?;
    let identity = observe_field_value(
        runtime,
        execution,
        snapshot,
        entity,
        layout.transition.entity_kind,
        &layout.transition.identity,
    )?;
    let Some(AspectValue::String(worth_foundational::facade::InternedString::Raw(identity))) =
        identity
    else {
        return Err(denial("original transition identity is absent"));
    };
    if identity.len() != 64 || !identity.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(denial("original transition identity is malformed"));
    }
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind: layout.transition.entity_kind,
        locator: layout.transition.identity.clone(),
        value: text(identity.clone()),
    });
    Ok((WorkflowTransitionLocator::new(entity, settlement), identity))
}
fn incoming(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    kind: worth_relational::facade::identity::KindId,
    child: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<EntityId, WorthQueryApplicationAttemptDenial> {
    let direction = WorthQueryApplicationAdjacencyDirection::Incoming;
    let relations = observe_adjacency(runtime, execution, snapshot, kind, child, direction, 2)?
        .ok_or_else(|| denial("original occurrence parent unavailable"))?;
    let [relation] = relations.as_slice() else {
        return Err(denial("original occurrence parent is not singular"));
    };
    let parent = relation.from;
    facts.push(WorthQueryApplicationObservedFact::Adjacency {
        relation_kind: kind,
        anchor: child,
        direction,
        maximum_work_units: 2,
        relations,
    });
    Ok(parent)
}

/// Reuses the native bounded history reader, without an Advance/cache/deadline path.
/// Positive evidence is reread from the exact record; absence keeps the history proof.
pub(in crate::domain_computation::primary_graph::application_attempt) fn recorded_approval(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    identity: &str,
    node: EntityId,
    occurrence: u64,
    maximum: usize,
    budget: worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget,
    compiled: &CompiledWorkflowDefinition,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<WorkflowTransitionLocator>, WorthQueryApplicationAttemptDenial> {
    let live = observe_adjacency(
        runtime,
        execution,
        snapshot,
        layout.live_instance_lineage_relation,
        instance,
        WorthQueryApplicationAdjacencyDirection::Outgoing,
        2,
    )?
    .ok_or_else(|| denial("original instance membership unavailable"))?;
    let basis = match live.len() {
        0 => history::HistoryBasis::Ended,
        1 => history::HistoryBasis::Close,
        _ => return Err(denial("original instance membership is not singular")),
    };
    let history = history::observe(
        execution,
        runtime,
        snapshot,
        layout,
        instance,
        basis,
        maximum,
        history::HistoryNodes::Compiled(compiled),
        budget,
    )?;
    let mut matching = history.transitions.iter().filter(|entry| {
        entry.identity == identity
            && entry.settlement.node() == node
            && entry.settlement.occurrence() == occurrence
    });
    let result = matching
        .next()
        .map(|entry| WorkflowTransitionLocator::new(entry.entity, entry.settlement));
    if matching.next().is_some() {
        return Err(denial("original approval occurrence is not singular"));
    }
    if result.is_none() {
        // Membership guards newly appended occurrences; every searched record's
        // original identity/node/occurrence guards why none matched.
        for entry in &history.transitions {
            settlement::observe_retained_transition(
                execution,
                runtime,
                snapshot,
                layout,
                WorkflowTransitionLocator::new(entry.entity, entry.settlement),
                facts,
            )?;
            exact(
                execution,
                runtime,
                snapshot,
                entry.entity,
                layout.transition.entity_kind,
                &layout.transition.identity,
                text(entry.identity.clone()),
                facts,
            )?;
        }
        facts.extend(history.facts);
    }
    Ok(result)
}

/// Immutable scope only. Current state, head, deadline and custody stay in command eligibility.
pub(in crate::domain_computation::primary_graph::application_attempt) fn original_scope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: &PublishedWorkflowInstanceRef,
    subject: EntityId,
    lineage: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let entity = instance.entity_id();
    let kind = layout.instance.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind,
    });
    for (locator, value) in [
        (&layout.instance.protocol_version, crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION),
        (&layout.instance.branch_occurrence, instance.started_occurrence()),
        (&layout.instance.subject_partition, u64::from(subject.partition_value())),
        (&layout.instance.subject_slot, subject.local_slot_value()),
        (&layout.instance.subject_generation, u64::from(subject.generation_value())),
    ] { exact(execution, runtime, snapshot, entity, kind, locator, AspectValue::UInt64(value), facts)?; }
    exact(
        execution,
        runtime,
        snapshot,
        entity,
        kind,
        &layout.instance.definition_content_identity,
        text(instance.definition_content_identity().to_string()),
        facts,
    )?;
    for (relation, expected) in [
        (
            layout.instance_definition_relation,
            instance.definition_entity_id(),
        ),
        (layout.instance_lineage_relation, lineage),
    ] {
        let targets = adjacency_with_kind(
            execution,
            runtime,
            snapshot,
            relation,
            entity,
            2,
            "original workflow scope unavailable",
            facts,
        )?;
        if targets.as_slice() != [expected] {
            return Err(denial("original workflow scope changed"));
        }
    }
    Ok(())
}
