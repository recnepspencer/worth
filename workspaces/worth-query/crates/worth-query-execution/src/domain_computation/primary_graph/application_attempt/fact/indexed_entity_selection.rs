use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
    DerivedIndexDefinition, DerivedIndexId,
};

use super::WorthQueryApplicationObservedFact;

pub(in crate::domain_computation::primary_graph) fn observe_indexed_entity_selection(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    entity_kind: KindId,
    locator: AspectFieldLocator,
    value: AspectValue,
    candidate_limit: usize,
) -> Option<WorthQueryApplicationObservedFact> {
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
    Some(WorthQueryApplicationObservedFact::IndexedEntitySelection {
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
    .is_some_and(|current| {
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
) -> Option<BoundedEntityFieldLookupOutcome> {
    let request = BoundedEntityFieldLookupRequest::new(
        snapshot.clone(),
        index_id,
        entity_kind,
        locator.clone(),
        value.clone(),
        candidate_limit,
    )
    .ok()?;
    let outcome = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production)
        .ok()?;
    (!outcome.overflowed()).then_some(outcome)
}

pub(super) fn remains_equal_with_work(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    definition: &std::sync::Arc<DerivedIndexDefinition>,
    entity_kind: KindId,
    locator: &AspectFieldLocator,
    value: &AspectValue,
    candidate_limit: usize,
    expected: &[EntityId],
) -> Option<(bool, usize)> {
    let current = bounded_entity_field_selection(
        runtime,
        snapshot,
        index_id,
        entity_kind,
        locator,
        value,
        candidate_limit,
    )?;
    let work = current.examined_entry_count().checked_add(1)?;
    Some((
        current.retain_definition().as_ref() == definition.as_ref()
            && current.candidate_entity_ids() == expected,
        work,
    ))
}
