//! Full row data and prior independent projection preimages, without services.
use super::*;
use crate::facade::symbols::StringInterner;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};
#[path = "tests/kind_oracle.rs"]
mod kind_oracle;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

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
fn controlled_full_region_matches_current_kind_preimages_and_truth() {
    let (input, symbols) = partition();
    let original = input.clone();
    let expected_digest = original.authoritative_content_digest(&symbols).unwrap();
    let expected_hashed = original.entity_arena.slot_count()
        + original.relation_arena.slot_count()
        + original
            .entity_arena
            .metadata_history
            .iter()
            .map(|row| row.len())
            .sum::<usize>()
        + original
            .relation_arena
            .metadata_history
            .iter()
            .map(|row| row.len())
            .sum::<usize>()
        + original.entity_arena.live_bitset.stored_word_count()
        + original.entity_arena.reclaimable_bitset.stored_word_count()
        + original.relation_arena.live_bitset.stored_word_count()
        + original
            .relation_arena
            .reclaimable_bitset
            .stored_word_count()
        + original
            .adjacency
            .iter()
            .chain(original.reverse_adjacency.iter())
            .map(|(_, set)| set.current_ids().len())
            .sum::<usize>();
    let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let prepared = PreparedResidentRegion::initial(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &ExecutionRequest::serial(&host),
        input,
        &symbols,
    )
    .unwrap();
    assert_eq!(prepared.content_digest, expected_digest);
    assert_eq!(prepared.content_values_hashed, expected_hashed as u64);
    let actual = &prepared
        .retention
        .capture_region()
        .unwrap()
        .unwrap()
        .partition;
    let old = &original;
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
        let projection = match basis {
            crate::storage::partition::AdjacencyKindBasis::Historical => {
                crate::storage::residency::kind_projection::canonical_history(
                    &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                    old,
                )
                .unwrap()
            }
            crate::storage::partition::AdjacencyKindBasis::Current => {
                crate::storage::residency::kind_projection::capture(
                    &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                    old,
                    basis,
                )
                .unwrap()
            }
        };
        // Independent canonical v3 byte grammar; no production digest/index helper.
        assert_eq!(kind_oracle::digest(&projection), *expected.bytes());
    }
    let local = RelationId::new(actual.partition_id, 2, 1);
    assert!(actual
        .adjacency
        .get(0)
        .unwrap()
        .historical_kind_ids(KindId(7))
        .iter()
        .any(|id| *id == local));
    assert!(actual
        .adjacency
        .get(2)
        .unwrap()
        .historical_kind_ids(KindId(7))
        .iter()
        .any(|id| *id == local));
    assert_eq!(
        actual.relation_arena.metadata_history_at(2).unwrap().len(),
        old.relation_arena.metadata_history_at(2).unwrap().len()
    );
    drop(prepared);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn controlled_full_commitment_clone_retains_only_its_actual_node_backing() {
    let (input, symbols) = partition();
    let unchanged_input = input.clone();
    let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let prepared = PreparedResidentRegion::initial(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &ExecutionRequest::serial(&host),
        input,
        &symbols,
    )
    .unwrap();
    let commitment = prepared
        .retention
        .capture_region()
        .unwrap()
        .unwrap()
        .commitment
        .as_ref()
        .unwrap()
        .clone();
    let expected = commitment.allocation_bytes();
    let original = commitment.digest(prepared.partition_id);
    assert!(expected > 0);
    assert!(budget.charged_memory_bytes() > expected);
    drop(prepared);
    assert_eq!(budget.charged_memory_bytes(), expected);
    assert_eq!(commitment.digest(unchanged_input.partition_id), original);
    drop(commitment);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(unchanged_input.entity_arena.slot_count(), 3);
}
