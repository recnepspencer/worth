//! Plain arena/partition DATA with an independent complete metadata oracle.
use super::*;
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::storage::partition::SparseAdjacencyTable;
use crate::storage::substrate::{
    EntityArena, EntityExtra, HistoryAppendFrontier, RelationArena, RelationEndpoints,
    RelationExtra, SlotInit,
};

pub(super) fn partitions() -> (
    PartitionState,
    PartitionState,
    RelationId,
    RecordHistoryOpening,
) {
    let partition_id = crate::identity::data::PartitionId::new(0);
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 8,
    };
    let mut full = PartitionState {
        partition_id,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: SparseAdjacencyTable::default(),
        reverse_adjacency: SparseAdjacencyTable::default(),
    };
    for slot in 0..3 {
        full.entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id,
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: EntityExtra::default(),
                },
                slot,
                1,
            )
            .unwrap();
        full.adjacency.ensure(slot, &policy);
        full.reverse_adjacency.ensure(slot, &policy);
    }
    let source = EntityId::new(partition_id, 0, 1);
    let first = EntityId::new(partition_id, 1, 1);
    let other = EntityId::new(partition_id, 2, 1);
    let relation = RelationId::new(partition_id, 13, 1);
    full.relation_arena
        .write_reserved_slot(
            SlotInit {
                partition_id,
                kind_id: KindId(2),
                version_id: VersionId(10),
                extra: RelationExtra::from_unbudgeted_parts(
                    Some(RelationEndpoints {
                        source,
                        target: first,
                    }),
                    None,
                ),
            },
            13,
            1,
        )
        .unwrap();
    full.adjacency
        .get_mut(0)
        .unwrap()
        .insert(KindId(2), relation);
    full.reverse_adjacency
        .get_mut(1)
        .unwrap()
        .insert(KindId(2), relation);
    update_target(&mut full, relation, other, VersionId(11));
    update_target(&mut full, relation, first, VersionId(12));
    let mut partial = full.clone();
    let physical = partial.relation_arena.physical_index(13).unwrap();
    let history = &partial.relation_arena.metadata_history[physical];
    let last = history[history.len() - 1].clone();
    partial
        .relation_arena
        .metadata_history
        .set(physical, vec![last].into());
    // An independently specified projection of omitted versions 10 and 11.
    // This DATA cannot construct a serving prefix or issue source custody.
    let opening = RecordHistoryOpening {
        frontier: HistoryAppendFrontier {
            siblings: vec![],
            last: None,
        },
        kind_commitment: [0; 32],
        historical_relations: vec![RegionRelationProjection {
            relation,
            kind: KindId(2),
            outgoing: BTreeSet::from([0]),
            incoming: BTreeSet::from([1, 2]),
        }],
    };
    (partial, full, relation, opening)
}

pub(super) fn reuse(partition: &mut PartitionState, relation: RelationId) {
    let reused = RelationId::new(
        relation.partition_id,
        relation.slot_index() as u64,
        relation.generation.0 + 1,
    );
    let slot = relation.slot_index();
    let extra = partition
        .relation_arena
        .get_slot(slot)
        .unwrap()
        .extra()
        .clone();
    let endpoints = extra.endpoints.clone().unwrap();
    partition
        .relation_arena
        .retire(slot, VersionId(1_001))
        .expect("fixture slot is materialized");
    partition
        .relation_arena
        .set_lifecycle_for_slot(slot, RecordLifecycleState::Reusable);
    partition
        .relation_arena
        .reset_slot(slot)
        .expect("fixture slot is materialized");
    partition
        .relation_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: relation.partition_id,
                kind_id: KindId(2),
                version_id: VersionId(1_002),
                extra,
            },
            slot,
            reused.generation.0,
        )
        .unwrap();
    for table in [&mut partition.adjacency, &mut partition.reverse_adjacency] {
        let adjacency = table.get_mut(endpoints.source.slot_index()).unwrap();
        adjacency.remove(KindId(2), &relation);
        adjacency.insert(KindId(2), reused);
    }
}

pub(super) fn update_target(
    partition: &mut PartitionState,
    relation: RelationId,
    target: EntityId,
    version: VersionId,
) {
    let mut extra = partition
        .relation_arena
        .get_slot(relation.slot_index())
        .unwrap()
        .extra()
        .clone();
    let endpoints = extra.endpoints.as_mut().unwrap();
    let old = endpoints.target;
    let source = endpoints.source;
    endpoints.target = target;
    partition
        .relation_arena
        .apply_extra_update(relation.slot_index(), extra, version)
        .expect("fixture slot is materialized");
    partition
        .reverse_adjacency
        .get_mut(old.slot_index())
        .unwrap()
        .remove_at(KindId(2), &relation, version);
    partition
        .reverse_adjacency
        .get_mut(target.slot_index())
        .unwrap()
        .insert_at(KindId(2), relation, version);
    partition
        .adjacency
        .get_mut(source.slot_index())
        .unwrap()
        .index_structural_revision(KindId(2), version);
}
