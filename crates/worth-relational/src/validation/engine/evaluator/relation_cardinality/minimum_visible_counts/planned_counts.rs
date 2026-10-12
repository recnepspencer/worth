//! Prepared minimum counts borrow the owner's unchanged maps.
use super::*;

pub(super) fn planned_visible_relation_counts<'scope>(
    context: &'scope InvariantExecutionContext<'_, '_>,
    contract: &LoweredCardinalityMinimumContract,
) -> VisibleRelationCountSnapshot<'scope> {
    let mut snapshot = VisibleRelationCountSnapshot::default();
    let Some(scope) = context.relation_integrity_scope(contract.relation_kind_id) else {
        return snapshot;
    };
    // Without deleted endpoints the prepared owner already holds exactly these
    // counts. Borrowing them needs no second scratch allocation.
    let borrow_counts = scope.deleted_entities.is_empty();
    if borrow_counts {
        if contract.source_min.is_some() {
            snapshot.source_counts = Cow::Borrowed(&scope.source_counts);
        }
        if contract.target_min.is_some() {
            snapshot.target_counts = Cow::Borrowed(&scope.target_counts);
        }
        if contract.pair_min.is_some() {
            snapshot.directed_pair_counts = Cow::Borrowed(&scope.directed_pair_counts);
        }
    }
    for (key, count) in &scope.source_counts {
        if !context.checkpoint(1) {
            return snapshot;
        }
        if borrow_counts || contract.source_min.is_none() {
            continue;
        }
        if !context.claim_scratch(
            (std::mem::size_of::<EntityReference>() + 3 * std::mem::size_of::<usize>()) as u64,
        ) {
            return snapshot;
        }
        snapshot.source_counts.to_mut().insert(key.clone(), *count);
    }
    for (key, count) in &scope.target_counts {
        if !context.checkpoint(1) {
            return snapshot;
        }
        if borrow_counts || contract.target_min.is_none() {
            continue;
        }
        if !context.claim_scratch(
            (std::mem::size_of::<EntityReference>() + 3 * std::mem::size_of::<usize>()) as u64,
        ) {
            return snapshot;
        }
        snapshot.target_counts.to_mut().insert(key.clone(), *count);
    }
    for (key, count) in &scope.directed_pair_counts {
        if !context.checkpoint(1) {
            return snapshot;
        }
        if borrow_counts || contract.pair_min.is_none() {
            continue;
        }
        if !context.claim_scratch(
            (2 * std::mem::size_of::<EntityReference>() + 3 * std::mem::size_of::<usize>()) as u64,
        ) {
            return snapshot;
        }
        snapshot
            .directed_pair_counts
            .to_mut()
            .insert(key.clone(), *count);
    }
    for edge in &scope.visible_edges {
        if !context.checkpoint(1) {
            return snapshot;
        }
        if scope.deleted_entities.contains(&edge.source)
            || scope.deleted_entities.contains(&edge.target)
        {
            subtract_relation_references(
                &mut snapshot,
                EntityReference::Existing(edge.source),
                EntityReference::Existing(edge.target),
            );
        }
    }
    for edge in &scope.planned_edges {
        if !context.checkpoint(1) {
            return snapshot;
        }
        let deleted_endpoint = [&edge.source, &edge.target].into_iter().any(|endpoint| {
            matches!(endpoint, EntityReference::Existing(id) if scope.deleted_entities.contains(id))
        });
        if deleted_endpoint {
            subtract_relation_references(&mut snapshot, edge.source.clone(), edge.target.clone());
        }
    }
    let state_view = context.state_view();
    for id in &scope.minimum_touched_entities {
        if !context.checkpoint(1) {
            return snapshot;
        }
        if scope.deleted_entities.contains(id) {
            continue;
        }
        if let Some(metadata) = state_view.entity_metadata(*id) {
            record_candidate_entity(
                context,
                contract,
                &mut snapshot,
                metadata.kind_id,
                EntityReference::Existing(*id),
            );
        }
    }
    for created in &scope.created_candidate_entities {
        if !context.checkpoint(1) {
            return snapshot;
        }
        record_candidate_entity(
            context,
            contract,
            &mut snapshot,
            created.kind_id,
            EntityReference::Created(created.clone()),
        );
    }
    snapshot
}
