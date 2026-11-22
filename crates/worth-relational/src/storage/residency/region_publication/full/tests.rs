//! Full row data and prior independent projection preimages, without services.
use super::*;
use crate::facade::symbols::StringInterner;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::substrate::{
    EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra, SlotInit,
};
#[path = "tests/kind_oracle.rs"]
mod kind_oracle;
#[path = "tests/locality_helpers.rs"]
mod locality_helpers;
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

#[test]
fn ordinary_one_slot_incremental_publication_does_not_scan_prior_partition_with_live_overlay() {
    use crate::storage::substrate::extra_delta_resolve_scans_on_current_thread;
    use locality_helpers::{
        assert_directed_ids, make_stored_extra, seed_stored_partition, stored_label_contract,
    };
    use worth_foundational::facade::StoredValueText;

    let contract = stored_label_contract();
    let registry = crate::schema::data::RelationalSchemaRegistry::default();
    let schema = crate::branch::RelationalBranchRootSchemaAuthority::capture(
        1,
        &registry,
        &registry.authority_snapshot(),
        crate::schema::data::DescriptorSemanticsVersion::default(),
        None,
    )
    .unwrap();
    let runtime_symbols = RuntimeSymbolTable::default();

    // Independently scale untouched population (8 -> 128) and prior history depth (2 -> 24).
    for (population, history_depth) in [(8_usize, 2_u64), (128_usize, 2_u64), (8_usize, 24_u64)] {
        let (base, existing_slot1_stored) =
            seed_stored_partition(&contract, population, history_depth);
        let budget = SerialMemoryBudget::new(Some(32 * 1024 * 1024));
        let host = SerialRequest::from_memory(budget, CancellationSource::new().token(), None);
        let execution = ExecutionRequest::serial(&host);
        let initial = PreparedResidentRegion::initial(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &execution,
            base,
            &StringInterner::default(),
        )
        .unwrap();
        let initial_region = initial.retention.capture_region().unwrap().unwrap();

        // 1. Fresh StoredValueText with an external live clone retaining its seal Arc
        //    (plus the borrowed overlay retained live): must incur ZERO prior-partition scans.
        let fresh_stored = StoredValueText::new(format!("fresh-{population}-{history_depth}"));
        let external_seal_holder = fresh_stored.clone();
        let mut overlay = initial_region.partition.clone();
        overlay
            .entity_arena
            .apply_extra_update(
                0,
                make_stored_extra(&contract, fresh_stored),
                VersionId(history_depth + 10),
            )
            .unwrap();
        let mut journal = PartitionMutationJournal::default();
        journal.entity_slots.insert(0);

        let scans_before = extra_delta_resolve_scans_on_current_thread();
        let updated = PreparedResidentRegion::incremental(
            &execution,
            RegionCaptureBasis {
                retention: &initial.retention,
                directory: &initial.directory,
                inventory: initial.inventory,
                unavailable_records: initial.unavailable_records,
                kind_commitment: initial.kind_commitment,
            },
            &overlay,
            &journal,
            schema.clone(),
            &runtime_symbols,
            None,
        )
        .unwrap();
        let scans_after = extra_delta_resolve_scans_on_current_thread();
        assert_eq!(overlay.entity_arena.slot_count(), population);
        assert!(!external_seal_holder.is_empty());
        assert_eq!(
            scans_after - scans_before,
            0,
            "fresh externally-cloned StoredValueText (pop={population}, hist={history_depth}) must never scan prior partition"
        );
        let actual = &updated.retention.capture_region().unwrap().unwrap().partition;
        let exp_new = assert_directed_ids(
            &initial_region.partition,
            actual,
            initial_region.sealed_watermarks,
        );
        assert_eq!(updated.new_payload_bytes, exp_new);
        assert_eq!(updated.inventory, actual.allocation_inventory().unwrap());
        drop((overlay, external_seal_holder));

        // 2. Separate existing-alias case: slot 0 adopts a clone of slot 1's pre-existing
        //    StoredValueText (seal born BEFORE initial was sealed). Exact directed IDs must
        //    omit the already-present seal from `introduced` and from `updated.inventory`.
        if population == 8 && history_depth == 2 {
            let mut alias_overlay = initial_region.partition.clone();
            alias_overlay
                .entity_arena
                .apply_extra_update(
                    0,
                    make_stored_extra(&contract, existing_slot1_stored),
                    VersionId(history_depth + 11),
                )
                .unwrap();
            let aliased = PreparedResidentRegion::incremental(
                &execution,
                RegionCaptureBasis {
                    retention: &initial.retention,
                    directory: &initial.directory,
                    inventory: initial.inventory,
                    unavailable_records: initial.unavailable_records,
                    kind_commitment: initial.kind_commitment,
                },
                &alias_overlay,
                &journal,
                schema.clone(),
                &runtime_symbols,
                None,
            )
            .unwrap();
            let alias_actual = &aliased.retention.capture_region().unwrap().unwrap().partition;
            let alias_new = assert_directed_ids(
                &initial_region.partition,
                alias_actual,
                initial_region.sealed_watermarks,
            );
            assert_eq!(aliased.new_payload_bytes, alias_new);
            assert_eq!(
                aliased.inventory,
                alias_actual.allocation_inventory().unwrap()
            );
        }
    }
}
