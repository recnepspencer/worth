//! Applied lookup support reuses the authentic predecessor selector and Native delta owner.
use super::super::super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use super::*;
use worth_relational::facade::{
    indexes::{
        AppliedEntityFieldLookupDenial as Stop, AppliedEntityFieldLookupOutcome,
        PreparedEntityFieldLookup,
    },
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};
type Candidate = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;
pub(in crate::domain_computation::primary_graph::application_attempt::fact) fn compare(
    fact: &WorthQueryApplicationObservedFact,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    before: &SnapshotHandle,
    left: Option<&Candidate>,
    right: &Candidate,
) -> Result<bool, Denial> {
    let WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        entity_kind,
        definition,
        locator,
        value,
        candidate_limit,
        candidates,
    } = fact
    else {
        return Err(unavailable());
    };
    if *candidate_limit == 0 || *candidate_limit == usize::MAX {
        return Err(unavailable());
    }
    let prepared = prepare(execution, runtime, before, *index_id, *entity_kind, locator)?;
    if prepared.retain_definition().as_ref() != definition.as_ref() {
        return Ok(false);
    }
    let right = lookup(execution, &prepared, value, *candidate_limit, right)?;
    if right.overflowed() {
        return Ok(false);
    }
    match left {
        None => Ok(right.candidate_entity_ids() == candidates),
        Some(left) => {
            let left = lookup(execution, &prepared, value, *candidate_limit, left)?;
            Ok(!left.overflowed() && left.candidate_entity_ids() == right.candidate_entity_ids())
        }
    }
}
fn lookup(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    prepared: &PreparedEntityFieldLookup,
    value: &AspectValue,
    limit: usize,
    pin: &Candidate,
) -> Result<AppliedEntityFieldLookupOutcome, Denial> {
    pin.lookup_applied_entity_field(
        execution,
        prepared,
        value,
        BoundedIndexParityMode::Production,
        limit,
    )
    .map_err(|cause| match cause {
        Stop::Execution(cause) => Denial::relational_read_stopped(cause, "applied indexed support"),
        Stop::PredecessorIndex(cause) => Denial::indexed_lookup_denied(
            Kind::MissingAuthoritativeFact,
            "applied indexed support",
            cause,
        ),
        _ => unavailable(),
    })
}
fn unavailable() -> Denial {
    Denial::new(Kind::MissingAuthoritativeFact, "applied indexed support")
}

fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    before: &SnapshotHandle,
    index: DerivedIndexId,
    kind: KindId,
    locator: &AspectFieldLocator,
) -> Result<PreparedEntityFieldLookup, Denial> {
    let view = runtime
        .read_truth()
        .project_snapshot(before)
        .ok_or_else(unavailable)?;
    runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index, kind, locator, execution)
        .map_err(|cause| {
            Denial::indexed_lookup_denied(
                Kind::MissingAuthoritativeFact,
                "applied indexed support",
                cause,
            )
        })
}

pub(in crate::domain_computation::primary_graph) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    before: &SnapshotHandle,
    candidate: &Candidate,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryIndexedSelectionRefusal> {
    let answer = (|| {
        let prepared = prepare(execution, runtime, before, index_id, entity_kind, &locator)?;
        let outcome = lookup(execution, &prepared, &value, candidate_limit, candidate)?;
        Ok::<_, Denial>((prepared.retain_definition(), outcome))
    })()
    .map_err(|cause| match cause.relational_execution_cause() {
        Some(cause) => WorthQueryIndexedSelectionRefusal::Execution(cause),
        None => WorthQueryIndexedSelectionRefusal::Unavailable,
    })?;
    let (definition, outcome) = answer;
    if outcome.overflowed() {
        return Err(WorthQueryIndexedSelectionRefusal::Overflowed);
    }
    Ok(WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates: outcome.into_candidate_entity_ids(),
    })
}
