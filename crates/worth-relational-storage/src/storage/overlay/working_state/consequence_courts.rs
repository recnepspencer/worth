//! Sequential graph writers must not disguise an earlier omitted graph.
use super::*;
use crate::identity::data::{EntityId, KindId, RelationId, VersionId};
use crate::storage::substrate::{
    EntityArena, EntityRecordKind, RecordKind, RelationArena, RelationEndpoints, RelationExtra,
    SlotInit,
};
use std::collections::BTreeMap;

#[test]
fn consequence_demand_recovers_untouched_adjacency_after_sequential_promotion() {
    let id = PartitionId(7);
    let policy = AdjacencyPolicy {
        backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let mut partition = PartitionState {
        partition_id: id,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(2),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    for _ in 0..2 {
        let _ = partition.entity_arena.push_slot(SlotInit {
            partition_id: id,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityRecordKind::empty_extra(),
        });
    }
    let _ = partition.relation_arena.push_slot(SlotInit {
        partition_id: id,
        kind_id: KindId(2),
        version_id: VersionId(1),
        extra: {
            let mut extra = RelationExtra::from_unbudgeted_parts(None, None);
            extra.endpoints = Some(RelationEndpoints {
                source: EntityId::new(id, 0, 1),
                target: EntityId::new(id, 1, 1),
            });
            extra
        },
    });
    let claim = RelationId::new(id, 0, 1);
    partition
        .adjacency
        .ensure(0, &policy)
        .insert(KindId(2), claim);
    partition
        .reverse_adjacency
        .ensure(1, &policy)
        .insert(KindId(2), claim);
    let base = BTreeMap::from([(id, partition)]);
    let mut earlier = WorkingState::from_touched_partitions(
        &base,
        [id],
        policy.clone(),
        PartitionCloneMode::EntityOnly,
    );
    let later = WorkingState::from_touched_partitions(
        &base,
        [id],
        policy,
        PartitionCloneMode::GraphSparseEntities,
    );
    earlier.adopt_sequential_apply_using(later, None).unwrap();
    assert!(earlier.partitions[&id].reverse_adjacency.get(1).is_none());
    earlier
        .materialize_consequence_records(
            &base,
            &[id].into(),
            &[EntityId::new(id, 1, 1)].into(),
            &Default::default(),
            |_, _| Ok::<_, ()>(()),
        )
        .unwrap();
    let recovered = &earlier.partitions[&id];
    assert_eq!(
        recovered
            .adjacency
            .get(0)
            .unwrap()
            .current_kind_ids(KindId(2))
            .to_vec(),
        [claim]
    );
    assert_eq!(
        recovered
            .reverse_adjacency
            .get(1)
            .unwrap()
            .current_kind_ids(KindId(2))
            .to_vec(),
        [claim]
    );
}
