//! A borrowed Head remains readable after later damage is proved.
use crate::branch::root::capture::reconstruction_fixture::PartitionContentOracle as _;
use worth_relational_storage::internal_port::storage::partition::adjacency::AdjacencySetFixturePort as _;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use super::mixed_history_tests::recapture;
use super::*;
use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::substrate::SlotInit;
use std::collections::BTreeSet;

#[test]
fn a_cached_head_guard_refuses_after_full_demand_proves_region_damage() {
    let fixture = region_with_history();
    let request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut cold = fixture.cold.clone();
    let mut members = fixture.source.members.clone();
    members.remove(&cold.history);
    let source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: cold.history,
        corrupt: AtomicBool::new(false),
    });
    cold.source = Some(source.clone());
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let retained = RelationalRegionRetention::new(cell, None);
    retained.demand_head_cached(execution, 1 << 24).unwrap();
    let accepted = retained.admitted_head_view().unwrap();
    let super::super::RelationalResidencyDemandFailure::StorageUnavailable(failure) =
        retained.demand_full(execution, 1 << 24).unwrap_err()
    else {
        panic!("missing named history must be a storage refusal")
    };
    assert_eq!(failure.cause, Cause::MissingMember);
    let reads = source.reads.load(Ordering::SeqCst);
    assert!(
        accepted.entity(fixture.entity).is_some(),
        "an already borrowed immutable view remains valid"
    );
    assert!(matches!(retained.demand_head_cached(execution, 1 << 24),
        Err(super::super::RelationalResidencyDemandFailure::StorageUnavailable(retry)) if retry == failure));
    assert!(
        retained.admitted_head_view().is_none(),
        "a probe must miss rather than accept a new damaged view"
    );
    assert!(matches!(retained.cell().capture_checkpoint(execution,
        fixture.cold.schema.as_ref(), fixture.cold.schema_carrier_digest,
        &fixture.cold.symbols.interner_snapshot(), fixture.cold.readmission.capture_members, None,
    ), Err(super::super::RelationalResidencyDemandFailure::StorageUnavailable(retry)) if retry == failure));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}

#[test]
fn head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse() {
    let fixture = region_with_history();
    let request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = fixture
        .cold
        .read_head(fixture.cold.source.as_ref().unwrap(), execution, 1 << 24)
        .unwrap();
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    let full = fixture
        .cold
        .read_full(
            fixture.cold.source.as_ref().unwrap(),
            &head,
            execution,
            1 << 24,
        )
        .unwrap();
    let mut partial = head.partition.clone();
    let mut oracle = full.partition.clone();
    assert_owned_history(&mut partial, &mut oracle);
    for partition in [&mut partial, &mut oracle] {
        update_target(
            partition,
            fixture.relation,
            fixture.entity,
            VersionId(1_000),
        );
    }
    assert_owned_history(&mut partial, &mut oracle);
    let reused = RelationId::new(
        fixture.relation.partition_id,
        fixture.relation.slot_index() as u64,
        fixture.relation.generation.0 + 1,
    );
    for partition in [&mut partial, &mut oracle] {
        let slot = fixture.relation.slot_index();
        let extra = partition
            .relation_arena
            .get_slot(slot)
            .unwrap()
            .extra()
            .clone();
        let endpoints = extra.endpoints.clone().unwrap();
        partition
            .relation_arena
            .retire_record_for_test(slot, VersionId(1_001))
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
            .write_reserved_slot_for_test(
                SlotInit {
                    partition_id: fixture.relation.partition_id,
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
            adjacency.remove_adjacent_id_for_test(KindId(2), &fixture.relation);
            adjacency.insert_adjacent_id_for_test(KindId(2), reused);
        }
    }
    assert_owned_history(&mut partial, &mut oracle);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 1);
}

#[test]
fn adopting_a_different_head_carries_its_owned_historical_projection_with_the_record() {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let fixture = region_with_history();
        let request = worth_harness::facade::execution_fixture::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&request);
        let head = fixture
            .cold
            .read_head(fixture.cold.source.as_ref().unwrap(), execution, 1 << 24)
            .unwrap();
        let full = fixture
            .cold
            .read_full(
                fixture.cold.source.as_ref().unwrap(),
                &head,
                execution,
                1 << 24,
            )
            .unwrap();
        let mut later = full.partition.clone();
        update_target(
            &mut later,
            fixture.relation,
            fixture.entity,
            VersionId(1_000),
        );
        let (cold, source) = recapture(&fixture.cold, &later);
        let selected = cold
            .read_head(cold.source.as_ref().unwrap(), execution, 1 << 24)
            .unwrap();
        let mut partial = head.partition.clone();
        let mut oracle = full.partition.clone();
        let slots = BTreeSet::from([fixture.relation.slot_index()]);
        partial
            .relation_arena
            .merge_slots_from_owned(&mut selected.partition.relation_arena.clone(), &slots);
        oracle
            .relation_arena
            .merge_slots_from_owned(&mut later.relation_arena.clone(), &slots);
        partial.adjacency = selected.partition.adjacency.clone();
        partial.reverse_adjacency = selected.partition.reverse_adjacency.clone();
        oracle.adjacency = later.adjacency.clone();
        oracle.reverse_adjacency = later.reverse_adjacency.clone();
        assert_owned_history(&mut partial, &mut oracle);
        assert_eq!(source.history_reads.load(Ordering::SeqCst), 0);
        let symbols = fixture.cold.symbols.interner_snapshot();
        let (commitment, _) = PartitionContentCommitment::capture_layered(
            fixture_lane,
            &worth_execution::ExecutionRequest::serial(
                &worth_harness::facade::execution_fixture::serial_request(),
            ),
            &partial,
            &symbols,
        )
        .unwrap();
        let kind_indexes = Arc::new(
            super::super::kind_incremental::RegionKindIndexes::capture_kind_indexes_for_test(
                fixture_lane,
                &partial,
            )
            .unwrap(),
        );
        let history_presence =
            super::super::history_presence::HistoryPresence::from_cold(fixture_lane, &partial);
        let owned = RelationalRegionResidency::resident_head(
            partial,
            commitment,
            kind_indexes,
            history_presence,
            ResidentHistoryReadmission::new(
                fixture.cold.schema.clone(),
                &fixture.cold.symbols,
                fixture.cold.readmission,
            ),
        );
        let reconstructed = full_demand(owned.cell(), 1 << 24).unwrap();
        assert_eq!(
            reconstructed
                .full_view()
                .partition()
                .authoritative_content_digest(&symbols)
                .unwrap(),
            oracle.authoritative_content_digest(&symbols).unwrap()
        );
        assert!(source.history_reads.load(Ordering::SeqCst) > 0,
        "Full promotion follows the adopted relation's own source as well as the original entity source");
    })
}

#[test]
fn a_full_target_adopting_head_prefixes_keeps_history_readmission() {
    let fixture = region_with_history();
    let request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = fixture
        .cold
        .read_head(fixture.cold.source.as_ref().unwrap(), execution, 1 << 24)
        .unwrap();
    let full = fixture
        .cold
        .read_full(
            fixture.cold.source.as_ref().unwrap(),
            &head,
            execution,
            1 << 24,
        )
        .unwrap();
    let readmission = ResidentHistoryReadmission::new(
        fixture.cold.schema.clone(),
        &fixture.cold.symbols,
        fixture.cold.readmission,
    );
    for metadata_first in [false, true] {
        let lease = RelationalRegionResidency::resident_uncommitted(full.partition.clone());
        let edit = if metadata_first {
            super::super::ResidentRegionEdit::from_full(lease)
        } else {
            super::super::ResidentRegionLease::Full(lease)
                .into_edit(readmission.clone())
                .0
        };
        let edited = edit
            .transform(readmission.clone(), |_| head.partition.clone())
            .freeze();
        assert!(
            matches!(edited, super::super::ResidentRegionLease::Head { .. }),
            "imported prefixes must not be mislabeled Full"
        );
        let head_lease = edited.head_lease();
        assert!(!head_lease.resident_region().history_presence.is_complete());
        let rebuilt = full_demand(head_lease.cell(), 1 << 24).unwrap();
        let symbols = fixture.cold.symbols.interner_snapshot();
        assert_eq!(
            rebuilt
                .full_view()
                .partition()
                .authoritative_content_digest(&symbols)
                .unwrap(),
            full.partition
                .authoritative_content_digest(&symbols)
                .unwrap(),
        );
    }
}

fn update_target(
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
        .apply_extra_update_for_test(relation.slot_index(), extra, version)
        .expect("fixture slot is materialized");
    partition
        .reverse_adjacency
        .get_mut(old.slot_index())
        .unwrap()
        .remove_adjacent_id_at_for_test(KindId(2), &relation, version);
    partition
        .reverse_adjacency
        .get_mut(target.slot_index())
        .unwrap()
        .insert_adjacent_id_at_for_test(KindId(2), relation, version);
    partition
        .adjacency
        .get_mut(source.slot_index())
        .unwrap()
        .index_structural_revision_for_test(KindId(2), version);
}

fn assert_owned_history(partial: &mut PartitionState, full: &mut PartitionState) {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        for partition in [&mut *partial, &mut *full] {
            for table in [&mut partition.adjacency, &mut partition.reverse_adjacency] {
                let slots = table.iter().map(|(&slot, _)| slot).collect::<Vec<_>>();
                for slot in slots {
                    table
                        .get_mut(slot)
                        .unwrap()
                        .reset_historical_kind_buckets_with_custody(None);
                }
            }
            super::super::kind_projection::rebuild_history_kind_buckets_for_test(
                fixture_lane,
                partition,
            )
            .unwrap();
        }
        assert_eq!(
            super::super::kind_projection::reconstruction::canonical_history_controlled(
                fixture_lane,
                partial,
                &mut |_, _| Ok(())
            )
            .unwrap(),
            super::super::kind_projection::reconstruction::canonical_history_controlled(
                fixture_lane,
                full,
                &mut |_, _| Ok(())
            )
            .unwrap()
        );
        assert_eq!(
            super::super::RelationalRegionKindCommitment::capture_kind_commitment_for_test(
                fixture_lane,
                partial
            )
            .unwrap(),
            super::super::RelationalRegionKindCommitment::capture_kind_commitment_for_test(
                fixture_lane,
                full
            )
            .unwrap()
        );
    })
}
