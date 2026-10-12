use super::*;

pub(in crate::domain_computation::primary_graph) fn recover_settled_live_membership(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    expected_identity: &str,
    expected_occurrence: u64,
) -> Result<(RelationId, Vec<WorthQueryApplicationObservedFact>), WorthQueryApplicationAttemptDenial>
{
    let kind = layout.transition.entity_kind;
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    }];
    exact(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.identity,
        text(expected_identity.to_owned()),
        &mut facts,
    )?;
    for (locator, expected) in [
        (
            &layout.transition.protocol_version,
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        (&layout.transition.occurrence, expected_occurrence),
        (
            &layout.transition.outcome,
            encode_transition_outcome(ApplicationWorkflowControlOutcome::Completed),
        ),
    ] {
        exact(execution,
            runtime,
            snapshot,
            transition,
            kind,
            locator,
            AspectValue::UInt64(expected),
            &mut facts,
        )?;
    }
    let partition = exact_u64(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_partition,
        &mut facts,
    )?;
    let slot = exact_u64(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_slot,
        &mut facts,
    )?;
    let generation = exact_u64(
        execution,
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_generation,
        &mut facts,
    )?;
    let partition = u32::try_from(partition)
        .map_err(|_| denial("settled live-membership partition is invalid"))?;
    let generation = u32::try_from(generation)
        .map_err(|_| denial("settled live-membership generation is invalid"))?;
    Ok((
        RelationId::new(PartitionId::new(partition), slot, generation),
        facts,
    ))
}
