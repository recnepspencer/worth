//! Observe actual retention owners copied by the authoritative edition spine.
use super::*;
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::storage::substrate::{EntityArena, RelationArena};

fn empty_region(id: PartitionId) -> RelationalRegionRetention {
    RelationalRegionRetention::resident(PartitionState {
        partition_id: id,
        adjacency_policy: AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    })
}

fn observed_write_copies(size: u32) -> usize {
    let previous = PartitionEdition::new(
        (0..size)
            .map(|id| (PartitionId(id), empty_region(PartitionId(id))))
            .collect(),
    );
    let before: Vec<_> = previous
        .cells()
        .map(|(id, region)| (id, Arc::strong_count(region.cell())))
        .collect();
    let mut successor = previous.clone();
    successor
        .map_mut()
        .insert(PartitionId(0), empty_region(PartitionId(0)));
    assert!(
        !Arc::ptr_eq(
            previous.partitions.get(&PartitionId(0)).unwrap().cell(),
            successor.partitions.get(&PartitionId(0)).unwrap().cell(),
        ),
        "the actual changed region must detach"
    );
    let mut copies = 0;
    for (id, old_owners) in before.into_iter().filter(|(id, _)| id.0 != 0) {
        let old = previous.partitions.get(&id).unwrap();
        let new = successor.partitions.get(&id).unwrap();
        assert!(
            Arc::ptr_eq(old.cell(), new.cell()),
            "untouched cells preserve identity"
        );
        copies += Arc::strong_count(old.cell())
            .checked_sub(old_owners)
            .unwrap();
    }
    copies.max(1)
}

#[test]
fn observed_small_write_shares_the_spine_as_partition_count_doubles() {
    let small = observed_write_copies(512);
    let large = observed_write_copies(1024);
    assert!(
        large * 2 < small * 3,
        "an observed write must copy changed paths, not every partition: {small} -> {large}"
    );
}
