use std::collections::{BTreeMap, BTreeSet};

use crate::config::data::AdjacencyBackend;
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{EntityArena, RelationArena};
use crate::storage::substrate::{EntityRecordKind, RecordKind, SlotInit};
use crate::storage::substrate::{RelationEndpoints, RelationExtra};

use super::super::{EntityWorkingSetLayout, PartitionCloneMode};
use super::WorkingState;

#[test]
fn touched_partition_working_state_only_clones_selected_partitions() {
    let policy = crate::config::data::AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let left = PartitionId(1);
    let right = PartitionId(2);
    let mut base = BTreeMap::new();
    base.insert(
        left,
        PartitionState {
            partition_id: left,
            adjacency_policy: policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(1),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        },
    );
    base.insert(
        right,
        PartitionState {
            partition_id: right,
            adjacency_policy: policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(1),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        },
    );

    let overlay =
        WorkingState::from_touched_partitions(&base, [right], policy, PartitionCloneMode::Full);

    assert!(!overlay.partitions.contains_key(&left));
    assert!(overlay.partitions.contains_key(&right));
    assert_eq!(overlay.touched_partition_count(), 1);
}

#[test]
fn touched_partition_working_state_preserves_candidate_layout_metadata() {
    let policy = crate::config::data::AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let partition = PartitionId(1);
    let mut base = BTreeMap::new();
    base.insert(
        partition,
        PartitionState {
            partition_id: partition,
            adjacency_policy: policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: EntityArena::with_capacity(8),
            relation_arena: RelationArena::with_capacity(0),
            adjacency: Default::default(),
            reverse_adjacency: Default::default(),
        },
    );

    let overlay = WorkingState::from_touched_partitions_with_layout(
        &base,
        [partition],
        policy,
        PartitionCloneMode::EntityOnly,
        EntityWorkingSetLayout::AoSoACandidate { chunk_width: 256 },
    );

    assert_eq!(
        overlay.entity_working_set_layout(),
        EntityWorkingSetLayout::AoSoACandidate { chunk_width: 256 }
    );
}

#[test]
fn sparse_entity_overlay_only_materializes_touched_slot_authoritative_metadata() {
    let policy = crate::config::data::AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let partition = PartitionId(1);
    let mut base_partition = PartitionState {
        partition_id: partition,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(2),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let _ = base_partition.entity_arena.push_slot(SlotInit {
        partition_id: partition,
        kind_id: KindId(1),
        version_id: VersionId(1),
        extra: EntityRecordKind::empty_extra(),
    });
    let _ = base_partition.entity_arena.push_slot(SlotInit {
        partition_id: partition,
        kind_id: KindId(1),
        version_id: VersionId(1),
        extra: EntityRecordKind::empty_extra(),
    });

    let mut base = BTreeMap::new();
    base.insert(partition, base_partition);
    let mut sparse_slots = BTreeMap::new();
    sparse_slots.insert(partition, [1usize].into_iter().collect());

    let overlay = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &base,
        [partition],
        policy,
        PartitionCloneMode::EntityOnly,
        EntityWorkingSetLayout::AoSoACandidate { chunk_width: 256 },
        Some(&sparse_slots),
        None,
    );

    let partition_state = overlay
        .partitions
        .get(&partition)
        .expect("partition present");
    assert!(partition_state.entity_arena.metadata_history[0].is_empty());
    assert_eq!(partition_state.entity_arena.metadata_history[1].len(), 1);
    assert!(partition_state.entity_arena.extra[0]
        .structural_fingerprint
        .is_none());
    assert!(partition_state.entity_arena.extra[0].lineage_id.is_none());
    assert!(partition_state.entity_arena.extra[0]
        .authoritative_aspect_state()
        .is_none());
}

#[test]
fn sparse_relation_overlay_keeps_relation_shape_without_full_authoritative_state_clone() {
    let policy = crate::config::data::AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let partition = PartitionId(11);
    let mut base_partition = PartitionState {
        partition_id: partition,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(2),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    let _ = base_partition.relation_arena.push_slot(SlotInit {
        partition_id: partition,
        kind_id: KindId(2),
        version_id: VersionId(1),
        extra: RelationExtra::from_unbudgeted_parts(
            Some(RelationEndpoints {
                source: EntityId::new(PartitionId(1), 0, 1),
                target: EntityId::new(PartitionId(2), 0, 1),
            }),
            None,
        ),
    });

    let mut base = BTreeMap::new();
    base.insert(partition, base_partition);
    let sparse_relation_partitions = BTreeSet::from([partition]);

    let overlay = WorkingState::from_touched_partitions_with_layout_and_sparse_slots(
        &base,
        [partition],
        policy,
        PartitionCloneMode::GraphSparseEntities,
        EntityWorkingSetLayout::CanonicalSoA,
        None,
        Some(&sparse_relation_partitions),
    );

    let partition_state = overlay
        .partitions
        .get(&partition)
        .expect("partition present");
    assert!(partition_state.relation_overlay_is_sparse);
    assert!(partition_state.relation_arena.metadata_history[0].is_empty());
    assert!(partition_state.relation_arena.extra[0].endpoints.is_none());
    assert!(partition_state.relation_arena.extra[0]
        .authoritative_aspect_state()
        .is_none());
}
