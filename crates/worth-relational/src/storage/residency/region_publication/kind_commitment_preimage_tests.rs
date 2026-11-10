//! The Full region kind commitment equals its independent projection preimage.
use super::PreparedResidentRegion;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};
use crate::symbols::data::StringInterner;
use sha2::{Digest, Sha256};

fn partition() -> (PartitionState, StringInterner) {
    let mut partition = PartitionState {
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
    };
    for slot in 0..3 {
        partition
            .entity_arena
            .write_reserved_slot(
                SlotInit {
                    partition_id: partition.partition_id,
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: EntityExtra::default(),
                },
                slot,
                1,
            )
            .unwrap();
        partition
            .adjacency
            .ensure(slot, &partition.adjacency_policy);
        partition
            .reverse_adjacency
            .ensure(slot, &partition.adjacency_policy);
    }
    let relation = RelationId::new(partition.partition_id, 2, 1);
    let mut extra = RelationExtra::default();
    extra.endpoints = Some(RelationEndpoints {
        source: EntityId::new(partition.partition_id, 0, 1),
        target: EntityId::new(partition.partition_id, 1, 1),
    });
    partition
        .relation_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: partition.partition_id,
                kind_id: KindId(7),
                version_id: VersionId(1),
                extra: extra.clone(),
            },
            2,
            1,
        )
        .unwrap();
    extra.endpoints.as_mut().unwrap().source = EntityId::new(partition.partition_id, 2, 1);
    partition
        .relation_arena
        .apply_extra_update(2, extra, VersionId(2))
        .unwrap();
    partition
        .adjacency
        .get_mut(2)
        .unwrap()
        .insert(KindId(7), relation);
    partition
        .reverse_adjacency
        .get_mut(1)
        .unwrap()
        .insert(KindId(7), relation);
    let peer = RelationId::new(PartitionId(2), 9, 3);
    partition
        .adjacency
        .get_mut(0)
        .unwrap()
        .insert(KindId(8), peer);
    partition
        .reverse_adjacency
        .get_mut(2)
        .unwrap()
        .insert(KindId(8), peer);
    let mut symbols = StringInterner::default();
    let later = symbols.intern("z-last").unwrap();
    let earlier = symbols.intern("a-first").unwrap();
    let physical = partition.entity_arena.physical_index(0).unwrap();
    partition
        .entity_arena
        .aspect_versions
        .set(physical, [(later, 9), (earlier, 4)].into_iter().collect());
    (partition, symbols)
}

#[test]
fn full_region_kind_commitment_matches_independent_projection_preimages() {
    let (input, symbols) = partition();
    let prepared = PreparedResidentRegion::initial(input, &symbols).unwrap();
    let region = &prepared
        .retention
        .capture_region()
        .unwrap()
        .unwrap()
        .partition;
    for (basis, expected) in [
        (
            crate::storage::partition::AdjacencyKindBasis::Current,
            prepared.kind_commitment.head,
        ),
        (
            crate::storage::partition::AdjacencyKindBasis::Historical,
            prepared.kind_commitment.history,
        ),
    ] {
        let projection =
            crate::storage::residency::kind_projection::capture(region, basis).unwrap();
        // The actual BTreeMap/BTreeSet projection and buffered compact
        // MessagePack preimage, independent of the commitment's own views.
        let bytes = rmp_serde::to_vec(&projection).unwrap();
        let mut hash = Sha256::new();
        hash.update(b"worth.relational.region-kind-authority.v2\0");
        hash.update(bytes);
        assert_eq!(<[u8; 32]>::from(hash.finalize()), *expected.bytes());
    }
}
