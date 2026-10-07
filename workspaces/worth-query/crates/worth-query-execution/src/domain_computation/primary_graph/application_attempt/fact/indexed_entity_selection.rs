use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupOutcome,
    BoundedEntityFieldLookupRequest, BoundedIndexParityMode, DerivedIndexDefinition,
    DerivedIndexId,
};

use super::WorthQueryApplicationObservedFact;

#[cfg(test)]
#[path = "indexed_entity_selection/denials_tests.rs"]
mod denials_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum IndexedSelectionReobserveDenial {
    UnexpectedFact,
    Lookup(BoundedEntityFieldLookupDenialKind),
    Overflow,
    WorkBudgetExceeded,
}

pub(in crate::domain_computation::primary_graph) fn observe_indexed_entity_selection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
) -> Option<WorthQueryApplicationObservedFact> {
    observe_checked(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
    )
    .ok()
}

fn observe_checked(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
) -> Result<WorthQueryApplicationObservedFact, IndexedSelectionReobserveDenial> {
    let outcome = bounded_entity_field_selection(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        &locator,
        &value,
        candidate_limit,
    )?;
    let definition = outcome.retain_definition();
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
) -> Result<BoundedEntityFieldLookupOutcome, IndexedSelectionReobserveDenial> {
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        index_id,
        entity_kind,
        locator.clone(),
        value.clone(),
        candidate_limit,
    )
    .map_err(|denial| IndexedSelectionReobserveDenial::Lookup(denial.kind()))?;
    let outcome = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .map_err(|denial| IndexedSelectionReobserveDenial::Lookup(denial.kind()))?;
    if outcome.overflowed() {
        Err(IndexedSelectionReobserveDenial::Overflow)
    } else {
        Ok(outcome)
    }
}

pub(in crate::domain_computation::primary_graph) fn reobserve(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    remaining_work: &mut usize,
) -> Result<WorthQueryApplicationObservedFact, IndexedSelectionReobserveDenial> {
    let WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
        ..
    } = fact
    else {
        return Err(IndexedSelectionReobserveDenial::UnexpectedFact);
    };
    let outcome = bounded_current_selection(
        runtime,
        snapshot,
        *index_id,
        *entity_kind,
        locator,
        value,
        *candidate_limit,
        remaining_work,
    )?;
    if outcome.overflowed() {
        return Err(IndexedSelectionReobserveDenial::Overflow);
    }
    Ok(WorthQueryApplicationObservedFact::IndexedEntitySelection {
        index_id: *index_id,
        definition: outcome.retain_definition(),
        entity_kind: *entity_kind,
        locator: locator.clone(),
        value: value.clone(),
        candidate_limit: *candidate_limit,
        candidates: outcome.into_candidate_entity_ids(),
    })
}

/// Bound the native probe by remaining work while retaining the selection's
/// original completeness contract. Sparse postings spend only examined rows.
fn bounded_current_selection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    candidate_limit: usize,
    remaining_work: &mut usize,
) -> Result<BoundedEntityFieldLookupOutcome, IndexedSelectionReobserveDenial> {
    // Validate the retained limit before narrowing it for this probe.
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        index_id,
        entity_kind,
        locator.clone(),
        value.clone(),
        candidate_limit,
    )
    .map_err(|denial| IndexedSelectionReobserveDenial::Lookup(denial.kind()))?;
    // The native request requires a positive row cap. With at most one work
    // unit left, refuse before probing rather than overspending on a row.
    let probe_limit = remaining_work
        .checked_sub(1)
        .filter(|rows| *rows > 0)
        .ok_or(IndexedSelectionReobserveDenial::WorkBudgetExceeded)?
        .min(candidate_limit);
    let request = if probe_limit == candidate_limit {
        request
    } else {
        BoundedEntityFieldLookupRequest::new(
            snapshot.clone(),
            index_id,
            entity_kind,
            locator.clone(),
            value.clone(),
            probe_limit,
        )
        .map_err(|denial| IndexedSelectionReobserveDenial::Lookup(denial.kind()))?
    };
    let observed = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production);
    let examined = match &observed {
        Ok(outcome) => outcome.examined_entry_count(),
        Err(denial) => denial.examined_entry_count(),
    };
    *remaining_work -= 1 + examined;
    let outcome =
        observed.map_err(|denial| IndexedSelectionReobserveDenial::Lookup(denial.kind()))?;
    if outcome.overflowed() && probe_limit < candidate_limit {
        return Err(IndexedSelectionReobserveDenial::WorkBudgetExceeded);
    }
    Ok(outcome)
}

pub(super) fn currentness(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    maximum_work: usize,
) -> Result<(bool, usize), super::WorthQuerySourceCurrentnessFailure> {
    let mut remaining_work = maximum_work;
    let current = currentness_with_remaining(fact, runtime, snapshot, &mut remaining_work)?;
    Ok((current, maximum_work - remaining_work))
}

/// The caller retains the bounded probe's actual debit even when lookup fails.
pub(in crate::domain_computation::primary_graph) fn currentness_with_remaining(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    remaining_work: &mut usize,
) -> Result<bool, super::WorthQuerySourceCurrentnessFailure> {
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
    let outcome = bounded_current_selection(
        runtime,
        snapshot,
        *index_id,
        *entity_kind,
        locator,
        value,
        *candidate_limit,
        remaining_work,
    )
    .map_err(|denial| match denial {
        IndexedSelectionReobserveDenial::WorkBudgetExceeded => Failure::WorkBudgetExceeded,
        _ => Failure::Unavailable,
    })?;
    Ok(!outcome.overflowed()
        && outcome.retain_definition().as_ref() == definition.as_ref()
        && outcome.candidate_entity_ids() == candidates)
}
