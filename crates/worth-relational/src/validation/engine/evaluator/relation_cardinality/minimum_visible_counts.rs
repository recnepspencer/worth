use std::collections::{BTreeMap, BTreeSet};

use crate::identity::data::{EntityId, KindId};
use crate::schema::data::LoweredCardinalityMinimumContract;
use crate::storage::substrate::{HistoricalMetadata, RelationArena, VersionedRelationMetadata};
use crate::transactions::data::EntityReference;

use super::super::super::context::InvariantExecutionContext;
use super::super::super::state_view::InvariantStateView;
use super::super::common::contract_candidate_kind_matches;

#[derive(Default)]
pub(super) struct VisibleRelationCountSnapshot {
    pub(super) source_counts: BTreeMap<EntityReference, usize>,
    pub(super) target_counts: BTreeMap<EntityReference, usize>,
    pub(super) directed_pair_counts: BTreeMap<(EntityReference, EntityReference), usize>,
    pub(super) candidate_source_entities: BTreeSet<EntityReference>,
    pub(super) candidate_target_entities: BTreeSet<EntityReference>,
    pub(super) relation_slot_scans: usize,
    pub(super) entity_slot_scans: usize,
}

/// One committed-current-version scan shared by all minimum registrations in
/// an invariant execution. Historical observations retain their versioned scan.
pub(crate) struct CurrentVersionMinimumIndex {
    entities: BTreeMap<KindId, Vec<EntityId>>,
    relations: BTreeMap<KindId, Vec<(EntityId, EntityId)>>,
    entity_slot_scans: usize,
    relation_slot_scans: usize,
}

impl CurrentVersionMinimumIndex {
    fn build(context: &InvariantExecutionContext<'_>) -> Self {
        let mut index = Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            entity_slot_scans: 0,
            relation_slot_scans: 0,
        };
        let state = context.state_view();
        for partition_id in state.state().partition_ids() {
            let Some(partition) = state.state().get_partition(partition_id) else {
                continue;
            };
            for slot in partition.entity_arena.live_bitset.iter_set_slots() {
                index.entity_slot_scans += 1;
                let Some(view) = partition.entity_arena.get_slot(slot) else {
                    continue;
                };
                let Some(kind_id) = view.kind_id() else {
                    continue;
                };
                index
                    .entities
                    .entry(kind_id)
                    .or_default()
                    .push(EntityId::new(partition_id, slot as u64, view.generation()));
            }
            for slot in partition.relation_arena.live_bitset.iter_set_slots() {
                index.relation_slot_scans += 1;
                let Some(view) = partition.relation_arena.get_slot(slot) else {
                    continue;
                };
                let (Some(kind_id), Some(endpoints)) =
                    (view.kind_id(), view.extra().endpoints.as_ref())
                else {
                    continue;
                };
                index
                    .relations
                    .entry(kind_id)
                    .or_default()
                    .push((endpoints.source, endpoints.target));
            }
        }
        context
            .metrics()
            .count_entity_slot_scans(index.entity_slot_scans);
        context
            .metrics()
            .count_relation_slot_scans(index.relation_slot_scans);
        index
    }
}

pub(super) fn visible_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
) -> VisibleRelationCountSnapshot {
    if context.merged_plan().is_some() {
        return planned_visible_relation_counts(context, contract);
    }
    let state_view = context.state_view();
    let mut snapshot = VisibleRelationCountSnapshot::default();

    if state_view.version_id() == context.current_version_id() {
        let mut built_scans = None;
        let index = context.current_version_minimum_index().get_or_init(|| {
            let index = CurrentVersionMinimumIndex::build(context);
            built_scans = Some((index.entity_slot_scans, index.relation_slot_scans));
            index
        });
        if let Some((entity_scans, relation_scans)) = built_scans {
            snapshot.entity_slot_scans = entity_scans;
            snapshot.relation_slot_scans = relation_scans;
        }
        if let Some(relations) = index.relations.get(&contract.relation_kind_id) {
            for &(source, target) in relations {
                record_existing_relation_endpoints(&mut snapshot, source, target);
            }
        }
        for kind_id in contract
            .candidate_source_kinds
            .iter()
            .chain(&contract.candidate_target_kinds)
            .copied()
            .collect::<BTreeSet<_>>()
        {
            if let Some(entities) = index.entities.get(&kind_id) {
                for &entity_id in entities {
                    record_candidate_entity(
                        contract,
                        &mut snapshot,
                        kind_id,
                        EntityReference::Existing(entity_id),
                    );
                }
            }
        }
        return snapshot;
    }

    for partition_id in state_view.state().partition_ids() {
        let Some(partition) = state_view.state().get_partition(partition_id) else {
            continue;
        };
        collect_historical_relation_counts(context, contract, partition, &mut snapshot);
        collect_historical_candidate_entities(
            context,
            contract,
            partition_id,
            partition,
            &mut snapshot,
        );
    }

    snapshot
}

fn planned_visible_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
) -> VisibleRelationCountSnapshot {
    let mut snapshot = VisibleRelationCountSnapshot::default();
    let Some(scope) = context.relation_integrity_scope(contract.relation_kind_id) else {
        return snapshot;
    };
    for (key, count) in &scope.source_counts {
        snapshot.source_counts.insert(key.entity_id.clone(), *count);
    }
    for (key, count) in &scope.target_counts {
        snapshot.target_counts.insert(key.entity_id.clone(), *count);
    }
    for (key, count) in &scope.directed_pair_counts {
        snapshot
            .directed_pair_counts
            .insert((key.source.clone(), key.target.clone()), *count);
    }
    for edge in &scope.visible_edges {
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
        let deleted_endpoint = [&edge.source, &edge.target].into_iter().any(|endpoint| {
            matches!(endpoint, EntityReference::Existing(id) if scope.deleted_entities.contains(id))
        });
        if deleted_endpoint {
            subtract_relation_references(&mut snapshot, edge.source.clone(), edge.target.clone());
        }
    }
    let state_view = context.state_view();
    for id in &scope.minimum_touched_entities {
        if scope.deleted_entities.contains(id) {
            continue;
        }
        if let Some(metadata) = state_view.entity_metadata(*id) {
            record_candidate_entity(
                contract,
                &mut snapshot,
                metadata.kind_id,
                EntityReference::Existing(*id),
            );
        }
    }
    for created in &scope.created_candidate_entities {
        record_candidate_entity(
            contract,
            &mut snapshot,
            created.kind_id,
            EntityReference::Created(created.clone()),
        );
    }
    snapshot
}

fn collect_historical_relation_counts(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.relation_arena.occupied_slots() {
        context.metrics().count_relation_slot_scans(1);
        snapshot.relation_slot_scans += 1;
        let Some(metadata) =
            visible_relation_metadata(&state_view, &partition.relation_arena, slot)
        else {
            continue;
        };
        if metadata.kind_id != contract.relation_kind_id {
            continue;
        }
        record_existing_relation_endpoints(
            snapshot,
            metadata.endpoints.source,
            metadata.endpoints.target,
        );
    }
}

fn collect_historical_candidate_entities(
    context: &InvariantExecutionContext<'_>,
    contract: &LoweredCardinalityMinimumContract,
    partition_id: crate::identity::data::PartitionId,
    partition: &crate::storage::overlay::PartitionState,
    snapshot: &mut VisibleRelationCountSnapshot,
) {
    let state_view = context.state_view();
    for slot in partition.entity_arena.occupied_slots() {
        context.metrics().count_entity_slot_scans(1);
        snapshot.entity_slot_scans += 1;
        let Some(metadata) =
            state_view.entity_metadata_at(&partition.entity_arena, partition_id, slot)
        else {
            continue;
        };
        record_candidate_entity(
            contract,
            snapshot,
            metadata.kind_id,
            EntityReference::Existing(metadata.entity_id),
        );
    }
}

fn record_existing_relation_endpoints(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) {
    record_relation_references(
        snapshot,
        EntityReference::Existing(source),
        EntityReference::Existing(target),
    );
}

fn record_relation_references(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: EntityReference,
    target: EntityReference,
) {
    *snapshot.source_counts.entry(source.clone()).or_insert(0) += 1;
    *snapshot.target_counts.entry(target.clone()).or_insert(0) += 1;
    *snapshot
        .directed_pair_counts
        .entry((source, target))
        .or_insert(0) += 1;
}

fn subtract_relation_references(
    snapshot: &mut VisibleRelationCountSnapshot,
    source: EntityReference,
    target: EntityReference,
) {
    decrement_count(&mut snapshot.source_counts, source.clone());
    decrement_count(&mut snapshot.target_counts, target.clone());
    decrement_count(&mut snapshot.directed_pair_counts, (source, target));
}

fn decrement_count<Key: Ord>(counts: &mut BTreeMap<Key, usize>, key: Key) {
    if let Some(count) = counts.get_mut(&key) {
        *count -= 1;
        if *count == 0 {
            counts.remove(&key);
        }
    }
}

fn record_candidate_entity(
    contract: &LoweredCardinalityMinimumContract,
    snapshot: &mut VisibleRelationCountSnapshot,
    kind_id: crate::identity::data::KindId,
    entity: EntityReference,
) {
    if contract_candidate_kind_matches(kind_id, &contract.candidate_source_kinds) {
        snapshot.candidate_source_entities.insert(entity.clone());
    }
    if contract_candidate_kind_matches(kind_id, &contract.candidate_target_kinds) {
        snapshot.candidate_target_entities.insert(entity);
    }
}

fn visible_relation_metadata<'state>(
    state_view: &InvariantStateView<'state>,
    arena: &'state RelationArena,
    slot: usize,
) -> Option<&'state VersionedRelationMetadata> {
    let history = arena.metadata_history_at(slot)?;
    let end = history.partition_point(|entry| entry.effective_at() <= state_view.version_id());
    (0..end).rev().map(|index| &history[index]).find(|entry| {
        entry.effective_at() <= state_view.version_id()
            && entry
                .retired_at()
                .is_none_or(|retired| state_view.version_id() < retired)
    })
}
