//! Physical fixture for read tests whose subject starts at a resident partition.
use super::PartitionState;
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::partition::AdjacencySet;
use crate::storage::substrate::{
    EntityArena, EntityRecordKind, RecordKind, RelationArena, RelationEndpoints, RelationExtra,
    SlotInit,
};

pub(crate) struct SmallRelationPartition {
    pub(crate) partition: PartitionState,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    pub(crate) relation: Option<RelationId>,
}

pub(crate) fn small_relation_partition(
    partition_id: PartitionId,
    linked: bool,
) -> SmallRelationPartition {
    let edges = [(0, 1, KindId(2))];
    let (partition, entities, relations) =
        relation_partition(partition_id, 2, if linked { &edges } else { &[] });
    SmallRelationPartition {
        partition,
        source: entities[0],
        target: entities[1],
        relation: relations.first().copied(),
    }
}

pub(crate) fn relation_partition(
    partition_id: PartitionId,
    entity_count: usize,
    edges: &[(usize, usize, KindId)],
) -> (PartitionState, Vec<EntityId>, Vec<RelationId>) {
    typed_relation_partition(
        partition_id,
        std::iter::repeat_n(KindId(1), entity_count),
        edges,
    )
}

pub(crate) fn typed_relation_partition(
    partition_id: PartitionId,
    entity_kinds: impl ExactSizeIterator<Item = KindId>,
    edges: &[(usize, usize, KindId)],
) -> (PartitionState, Vec<EntityId>, Vec<RelationId>) {
    let entity_count = entity_kinds.len();
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let mut partition = PartitionState {
        partition_id,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(entity_count),
        relation_arena: RelationArena::with_capacity(edges.len()),
        adjacency: vec![AdjacencySet::new(&policy); entity_count].into(),
        reverse_adjacency: vec![AdjacencySet::new(&policy); entity_count].into(),
    };
    let entities = entity_kinds
        .map(|kind_id| {
            let (slot, generation, _) = partition.entity_arena.push_slot(SlotInit {
                partition_id,
                kind_id,
                version_id: VersionId(1),
                extra: EntityRecordKind::empty_extra(),
            });
            EntityId::new(partition_id, slot as u64, generation)
        })
        .collect::<Vec<_>>();
    let mut relations = Vec::with_capacity(edges.len());
    for &(from, to, kind_id) in edges {
        let source = entities[from];
        let target = entities[to];
        let (slot, generation, _) = partition.relation_arena.push_slot(SlotInit {
            partition_id,
            kind_id,
            version_id: VersionId(1),
            extra: RelationExtra::from_unbudgeted_parts(
                Some(RelationEndpoints { source, target }),
                None,
            ),
        });
        let relation = RelationId::new(partition_id, slot as u64, generation);
        partition
            .adjacency
            .get_mut(source.slot_index())
            .unwrap()
            .insert(kind_id, relation);
        partition
            .reverse_adjacency
            .get_mut(target.slot_index())
            .unwrap()
            .insert(kind_id, relation);
        relations.push(relation);
    }
    (partition, entities, relations)
}
