//! Storage-native metadata and adjacency stimuli; no application runtime.
use super::*;
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::substrate::{
    EntityArena, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};

pub(super) fn partition(count: usize) -> PartitionState {
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let mut partition = PartitionState {
        partition_id: PartitionId(7),
        adjacency_policy: policy,
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    for _ in 0..4 {
        partition.entity_arena.push_slot(SlotInit {
            partition_id: partition.partition_id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: Default::default(),
        });
    }
    for slot in 0..4 {
        partition
            .adjacency
            .ensure(slot, &partition.adjacency_policy);
        partition
            .reverse_adjacency
            .ensure(slot, &partition.adjacency_policy);
    }
    for slot in 0..count {
        create(&mut partition, slot, 1);
    }
    partition
}
fn create(partition: &mut PartitionState, slot: usize, version: u64) {
    let kind = KindId(7 + slot as u32 % 2);
    let endpoints = endpoints(1);
    let (actual, generation, _) = partition.relation_arena.push_slot(SlotInit {
        partition_id: partition.partition_id,
        kind_id: kind,
        version_id: VersionId(version),
        extra: RelationExtra {
            endpoints: Some(endpoints.clone()),
            authoritative_aspect_state: None,
        },
    });
    assert_eq!(actual, slot);
    insert(
        partition,
        RelationId::new(partition.partition_id, slot as u64, generation),
        kind,
        &endpoints,
    );
}
fn endpoints(target: usize) -> RelationEndpoints {
    RelationEndpoints {
        source: EntityId::new(PartitionId(7), 0, 1),
        target: EntityId::new(PartitionId(7), target as u64, 1),
    }
}
fn insert(
    partition: &mut PartitionState,
    id: RelationId,
    kind: KindId,
    endpoints: &RelationEndpoints,
) {
    partition
        .adjacency
        .ensure(endpoints.source.slot_index(), &partition.adjacency_policy)
        .insert(kind, id);
    partition
        .reverse_adjacency
        .ensure(endpoints.target.slot_index(), &partition.adjacency_policy)
        .insert(kind, id);
}

pub(super) fn edit(
    partition: &mut PartitionState,
    slot: usize,
    step: usize,
) -> PartitionMutationJournal {
    let view = partition.relation_arena.get_slot(slot).unwrap();
    let id = RelationId::new(partition.partition_id, slot as u64, view.generation());
    let kind = view.kind_id().unwrap();
    let old = view.extra().endpoints.clone().unwrap();
    let new = endpoints(1 + step % 3);
    partition
        .adjacency
        .get_mut(old.source.slot_index())
        .unwrap()
        .remove(kind, &id);
    partition
        .reverse_adjacency
        .get_mut(old.target.slot_index())
        .unwrap()
        .remove(kind, &id);
    partition
        .relation_arena
        .apply_extra_update(
            slot,
            RelationExtra {
                endpoints: Some(new.clone()),
                authoritative_aspect_state: None,
            },
            VersionId(step as u64 + 2),
        )
        .unwrap();
    if step % 5 == 1 {
        partition
            .relation_arena
            .retire(slot, VersionId(step as u64 + 2))
            .unwrap();
    } else {
        partition
            .relation_arena
            .set_lifecycle_for_slot(slot, crate::storage::data::RecordLifecycleState::Live);
        partition.relation_arena.live_bitset.set(slot, true);
        partition.relation_arena.reclaimable_bitset.set(slot, false);
        let physical = partition.relation_arena.physical_index(slot).unwrap();
        partition.relation_arena.retired_at.set(physical, None);
        insert(partition, id, kind, &new);
    }
    PartitionMutationJournal {
        entity_slots: Default::default(),
        relation_slots: [slot].into(),
        adjacency_slots: [old.source.slot_index(), new.source.slot_index()].into(),
        reverse_adjacency_slots: [old.target.slot_index(), new.target.slot_index()].into(),
    }
}

pub(super) fn restore(
    partition: &mut PartitionState,
    source: &PartitionState,
    slot: usize,
) -> PartitionMutationJournal {
    let old = partition.relation_arena.get_slot(slot).unwrap();
    let old_endpoints = old.extra().endpoints.clone();
    if let (Some(endpoints), Some(kind)) = (&old_endpoints, old.kind_id()) {
        let relation = RelationId::new(partition.partition_id, slot as u64, old.generation());
        partition
            .adjacency
            .get_mut(endpoints.source.slot_index())
            .unwrap()
            .remove(kind, &relation);
        partition
            .reverse_adjacency
            .get_mut(endpoints.target.slot_index())
            .unwrap()
            .remove(kind, &relation);
    }
    partition
        .relation_arena
        .merge_slots_from_owned(&mut source.relation_arena.clone(), &[slot].into());
    let new = partition.relation_arena.get_slot(slot).unwrap();
    let new_endpoints = new.extra().endpoints.clone();
    if let (Some(endpoints), Some(kind)) = (&new_endpoints, new.kind_id()) {
        let relation = RelationId::new(partition.partition_id, slot as u64, new.generation());
        insert(partition, relation, kind, endpoints);
    }
    journal(slot, old_endpoints.iter().chain(new_endpoints.iter()))
}
pub(super) fn suspend(partition: &mut PartitionState, slot: usize) -> PartitionMutationJournal {
    let view = partition.relation_arena.get_slot(slot).unwrap();
    let relation = RelationId::new(partition.partition_id, slot as u64, view.generation());
    let kind = view.kind_id().unwrap();
    let endpoints = view.extra().endpoints.clone().unwrap();
    partition
        .adjacency
        .get_mut(endpoints.source.slot_index())
        .unwrap()
        .remove(kind, &relation);
    partition
        .reverse_adjacency
        .get_mut(endpoints.target.slot_index())
        .unwrap()
        .remove(kind, &relation);
    partition
        .relation_arena
        .suspend_materialization(slot)
        .unwrap();
    journal(slot, std::iter::once(&endpoints))
}
fn journal<'a>(
    slot: usize,
    endpoints: impl Iterator<Item = &'a RelationEndpoints>,
) -> PartitionMutationJournal {
    let mut journal = PartitionMutationJournal {
        relation_slots: [slot].into(),
        ..Default::default()
    };
    for endpoint in endpoints {
        journal.adjacency_slots.insert(endpoint.source.slot_index());
        journal
            .reverse_adjacency_slots
            .insert(endpoint.target.slot_index());
    }
    journal
}
