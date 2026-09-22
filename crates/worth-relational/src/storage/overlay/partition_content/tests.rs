use super::*;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, SlotInit, StorageAllocationObservation,
};

#[test]
fn one_record_update_hashes_three_values_regardless_of_population_or_history() {
    for (records, history) in [(1, 1), (128, 64), (4096, 1024)] {
        let mut old = partition();
        for slot in 0..records {
            old.entity_arena
                .write_reserved_slot(
                    SlotInit {
                        partition_id: old.partition_id,
                        kind_id: KindId(1),
                        version_id: VersionId(1),
                        extra: EntityExtra::default(),
                    },
                    slot,
                    1,
                )
                .unwrap();
        }
        for version in 2..=history {
            old.entity_arena
                .apply_extra_update(0, EntityExtra::default(), VersionId(version));
        }
        let symbols = StringInterner::default();
        let (retained, _) = PartitionContentCommitment::capture(&old, &symbols, None).unwrap();
        let before_digest = retained.digest(old.partition_id);
        let mut next = old.clone();
        next.entity_arena
            .apply_extra_update(0, EntityExtra::default(), VersionId(history + 1));
        let journal = PartitionMutationJournal {
            entity_slots: [0].into_iter().collect(),
            ..Default::default()
        };
        let (incremental, hashed) =
            PartitionContentCommitment::capture(&next, &symbols, Some((&retained, &old, &journal)))
                .unwrap();
        assert_eq!(
            hashed, 3,
            "one record, retired history tail, new history entry"
        );
        assert_eq!(
            incremental.digest(next.partition_id),
            next.authoritative_content_digest(&symbols).unwrap()
        );
        assert_eq!(retained.digest(old.partition_id), before_digest);
        assert_ne!(incremental.digest(next.partition_id), before_digest);
        assert_eq!(
            next.allocation_inventory_since(&old, old.allocation_inventory()),
            next.allocation_inventory()
        );
        let mut bytes = Bytes::default();
        incremental.visit_allocations(&mut bytes);
        assert_eq!(bytes.0, incremental.allocation_bytes());
    }
}

fn partition() -> PartitionState {
    PartitionState {
        partition_id: PartitionId(1),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}

#[derive(Default)]
struct Bytes(u64);
impl StorageAllocationVisitor for Bytes {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        self.0 += allocation.bytes;
        true
    }
}
