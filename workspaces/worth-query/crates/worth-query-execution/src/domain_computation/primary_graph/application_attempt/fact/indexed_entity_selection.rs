use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
    DerivedIndexDefinition, DerivedIndexId,
};

use super::WorthQueryApplicationObservedFact;

/// Why an indexed lookup yields no selection fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryIndexedSelectionRefusal {
    /// More entities hold the value than the lookup's candidate limit.
    Overflowed,
    /// The index could not answer at the snapshot.
    Unavailable,
}

pub(in crate::domain_computation::primary_graph) fn observe_indexed_entity_selection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryIndexedSelectionRefusal> {
    observe_examined(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidate_limit,
    )
    .map(|(selection, _)| selection)
}

/// The entities holding `value` at the snapshot, for a reader that records
/// no fact: a merge's unique lookup reads the candidates only.
pub(in crate::domain_computation::primary_graph) fn observe_indexed_candidates(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    candidate_limit: usize,
) -> Result<Vec<EntityId>, WorthQueryIndexedSelectionRefusal> {
    bounded_entity_field_selection(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
    )
    .map(BoundedEntityFieldLookupOutcome::into_candidate_entity_ids)
}

/// The selection as the snapshot holds it, with the index entries its lookup
/// examined. The lookup reads at most `lookup_limit` candidates, which is the
/// recorded `candidate_limit` unless a caller's work caps it lower.
#[allow(clippy::too_many_arguments)]
fn observe_examined(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
    lookup_limit: usize,
) -> Result<(WorthQueryApplicationObservedFact, usize), WorthQueryIndexedSelectionRefusal> {
    let outcome = bounded_entity_field_selection(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        &locator,
        &value,
        lookup_limit,
    )?;
    let examined = outcome.examined_entry_count();
    let definition = outcome.retain_definition();
    let selection = WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates: outcome.into_candidate_entity_ids(),
    };
    Ok((selection, examined))
}

pub(super) fn remains_equal(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    definition: &std::sync::Arc<DerivedIndexDefinition>,
    entity_kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    candidate_limit: usize,
    expected: &[EntityId],
) -> bool {
    bounded_entity_field_selection(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
    )
    .is_ok_and(|current| {
        current.retain_definition().as_ref() == definition.as_ref()
            && current.candidate_entity_ids() == expected
    })
}

fn bounded_entity_field_selection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    candidate_limit: usize,
) -> Result<BoundedEntityFieldLookupOutcome, WorthQueryIndexedSelectionRefusal> {
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        index_id,
        entity_kind,
        locator.clone(),
        value.clone(),
        candidate_limit,
    )
    .map_err(|_| WorthQueryIndexedSelectionRefusal::Unavailable)?;
    let outcome = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|_| WorthQueryIndexedSelectionRefusal::Unavailable)?;
    if outcome.overflowed() {
        return Err(WorthQueryIndexedSelectionRefusal::Overflowed);
    }
    Ok(outcome)
}

/// Why the selection could not be observed again.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum IndexedReobservation {
    /// The work cap stopped the lookup below the recorded candidate limit.
    Unpaid,
    /// The snapshot no longer yields the selection within its limit.
    Unavailable,
}

/// The selection observed again, reading at most `maximum_work - 1` index
/// entries. A selection the cap cut short of its recorded limit is unpaid.
pub(in crate::domain_computation::primary_graph) fn reobserve(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_work: usize,
) -> Result<(WorthQueryApplicationObservedFact, usize), IndexedReobservation> {
    let WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
        ..
    } = fact
    else {
        return Err(IndexedReobservation::Unavailable);
    };
    let lookup_limit =
        lookup_limit(*candidate_limit, maximum_work).ok_or(IndexedReobservation::Unpaid)?;
    match observe_examined(
        runtime,
        snapshot,
        *index_id,
        *entity_kind,
        locator.clone(),
        value.clone(),
        *candidate_limit,
        lookup_limit,
    ) {
        Ok(observed) => Ok(observed),
        Err(WorthQueryIndexedSelectionRefusal::Overflowed) if lookup_limit < *candidate_limit => {
            Err(IndexedReobservation::Unpaid)
        }
        Err(
            WorthQueryIndexedSelectionRefusal::Overflowed
            | WorthQueryIndexedSelectionRefusal::Unavailable,
        ) => Err(IndexedReobservation::Unavailable),
    }
}

/// The candidates a lookup paid `maximum_work` may read: one unit is the
/// lookup itself. `None` when the work cannot pay for a candidate the
/// recorded limit allows.
fn lookup_limit(candidate_limit: usize, maximum_work: usize) -> Option<usize> {
    let affordable = maximum_work.checked_sub(1)?;
    let limit = candidate_limit.min(affordable);
    (limit > 0 || candidate_limit == 0).then_some(limit)
}

pub(super) fn currentness(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_work: usize,
) -> Result<(bool, usize), super::WorthQuerySourceCurrentnessFailure> {
    use super::WorthQuerySourceCurrentnessFailure as Failure;
    let WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        definition,
        entity_kind,
        locator,
        value,
        candidate_limit,
        candidates,
    } = fact
    else {
        return Err(Failure::Unavailable);
    };
    let lookup_limit =
        lookup_limit(*candidate_limit, maximum_work).ok_or(Failure::WorkBudgetExceeded)?;
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        *index_id,
        *entity_kind,
        locator.clone(),
        value.clone(),
        lookup_limit,
    )
    .map_err(|_| Failure::Unavailable)?;
    let outcome = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|_| Failure::Unavailable)?;
    // More candidates than a capped lookup may read can still fit the
    // recorded limit, so the cap gives no answer.
    if outcome.overflowed() && lookup_limit < *candidate_limit {
        return Err(Failure::WorkBudgetExceeded);
    }
    Ok((
        !outcome.overflowed()
            && outcome.retain_definition().as_ref() == definition.as_ref()
            && outcome.candidate_entity_ids() == candidates,
        1 + outcome.examined_entry_count(),
    ))
}
