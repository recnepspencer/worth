use super::{
    denial, observed_optional_text, observed_u64, EntityId, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationObservedFact, WorthQueryWorkflowLayout,
};

pub(super) fn read_global_actions(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    definition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<
    Box<[worth_query_installation::facade::WorthQueryWorkflowGlobalActionDependency]>,
    WorthQueryApplicationAttemptDenial,
> {
    let fields = &layout.definition;
    let Some(encoded) = observed_optional_text(
        execution,
        runtime,
        snapshot,
        definition,
        fields.entity_kind,
        &fields.global_actions,
        facts,
    )?
    else {
        return Ok(Box::new([]));
    };
    let maximum_actions = observed_u64(
        execution,
        runtime,
        snapshot,
        definition,
        fields.entity_kind,
        &fields.maximum_effects,
        facts,
    )?;
    let maximum_bytes = observed_u64(
        execution,
        runtime,
        snapshot,
        definition,
        fields.entity_kind,
        &fields.maximum_canonical_bytes,
        facts,
    )?;
    super::super::super::global_actions::decode(
        &encoded,
        usize::try_from(maximum_actions).map_err(|_| denial("global action bound is invalid"))?,
        usize::try_from(maximum_bytes)
            .map_err(|_| denial("global action byte bound is invalid"))?,
    )
    .ok_or_else(|| denial("published global action contract is invalid"))
}
