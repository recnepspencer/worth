//! Owned historical kind truth survives Head edits without old-version reads.
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
    let request = crate::test_execution_policy::serial_request();
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
    cold.source = source.clone();
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
    assert!(matches!(retained.cell().capture_checkpoint(
        &fixture.cold.schema, fixture.cold.schema_carrier_digest,
        &fixture.cold.symbols.interner_snapshot(), None,
    ), Err(retry) if retry == failure));
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}

#[test]
fn head_prefixes_rebuild_owned_history_after_endpoint_edits_and_generation_reuse() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
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
        partition.relation_arena.retire(slot, VersionId(1_001));
        partition
            .relation_arena
            .set_lifecycle_for_slot(slot, RecordLifecycleState::Reusable);
        partition.relation_arena.reset_slot(slot);
        partition
            .relation_arena
            .write_reserved_slot(
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
            adjacency.remove(KindId(2), &fixture.relation);
            adjacency.insert(KindId(2), reused);
        }
    }
    assert_owned_history(&mut partial, &mut oracle);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 1);
}

#[test]
fn adopting_a_different_head_carries_its_owned_historical_projection_with_the_record() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
    let mut later = full.partition.clone();
    update_target(
        &mut later,
        fixture.relation,
        fixture.entity,
        VersionId(1_000),
    );
    let (cold, source) = recapture(&fixture.cold, &later);
    let selected = region_load::load_head(&cold, execution, 1 << 24).unwrap();
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
    let (commitment, _) = PartitionContentCommitment::capture_layered(&partial, &symbols).unwrap();
    let owned = RelationalRegionResidency::resident_head(
        partial,
        commitment,
        fixture.cold.schema.clone(),
        &fixture.cold.symbols,
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
        .apply_extra_update(relation.slot_index(), extra, version);
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

fn assert_owned_history(partial: &mut PartitionState, full: &mut PartitionState) {
    for partition in [&mut *partial, &mut *full] {
        for (_, adjacency) in partition
            .adjacency
            .iter_mut()
            .chain(partition.reverse_adjacency.iter_mut())
        {
            adjacency.reset_historical_kind_buckets();
        }
        super::super::rebuild_region_history_kind_buckets(partition).unwrap();
    }
    assert_eq!(
        super::super::kind_projection::canonical_history(partial).unwrap(),
        super::super::kind_projection::canonical_history(full).unwrap()
    );
    assert_eq!(
        super::super::RelationalRegionKindCommitment::capture(partial).unwrap(),
        super::super::RelationalRegionKindCommitment::capture(full).unwrap()
    );
}
