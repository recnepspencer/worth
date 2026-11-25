//! A commit's live and cancelled evidence replays as one contiguous append
//! block, in any live order; cancellations burn exactly what live did (H0, D2).
use super::*;
use crate::identity::data::{EntityId, KindId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::primitives::{EntitySpec, RecordRef};

const P: PartitionId = PartitionId(7);
const E: RecordAllocationClass = RecordAllocationClass::Entity;

fn entity(slot: u64, generation: u32) -> RecordRef {
    RecordRef::Entity(EntityId::new(P, slot, generation))
}
fn live(ordinal: u64, slot: u64) -> CanonicalRecordAllocation {
    CanonicalRecordAllocation::with_origin(
        ordinal,
        entity(slot, 1),
        RecordAllocationOrigin::AppendFrontier,
    )
}
fn cancelled(slot: u64) -> CancelledRecordAllocation {
    CancelledRecordAllocation::new(entity(slot, 1), RecordAllocationOrigin::AppendFrontier)
}
fn replay(authority: &RecordIdentitySubsystem, count: usize) -> Vec<usize> {
    let mut pending = authority.begin_allocations();
    let slots = (0..count)
        .map(|_| pending.reserve(E, P).unwrap().slot)
        .collect();
    pending.finish_mutation().unwrap();
    pending.commit();
    slots
}
fn broken(expected_slot: usize, observed_slot: usize) -> RecordAllocationDenial {
    RecordAllocationDenial::ReplayEvidenceBlockBroken {
        class: E,
        partition_id: P,
        expected_slot,
        observed_slot,
    }
}

#[test]
fn live_append_evidence_replays_in_plan_order_inside_its_block() {
    let authority = RecordIdentitySubsystem::default();
    authority
        .stage_replay_allocations(vec![live(0, 1), live(1, 0)], Vec::new())
        .unwrap();
    assert_eq!(replay(&authority, 2), vec![1, 0]);
    assert_eq!(authority.frontier_snapshot(), vec![(E, P, 2)]);
}

#[test]
fn a_hole_a_duplicate_or_a_slot_behind_the_frontier_is_refused_unchanged() {
    let authority = RecordIdentitySubsystem::default();
    let refuse = |live_slots: Vec<CanonicalRecordAllocation>,
                  gone: Vec<CancelledRecordAllocation>| {
        authority
            .stage_replay_allocations(live_slots, gone)
            .unwrap_err()
    };
    assert_eq!(
        refuse(vec![live(0, 0), live(1, 2)], Vec::new()),
        broken(1, 2)
    );
    assert_eq!(refuse(vec![live(0, 0)], vec![cancelled(0)]), broken(1, 0));
    authority.restore_frontier(E, P, 3);
    assert_eq!(
        refuse(vec![live(0, 1), live(1, 2)], Vec::new()),
        broken(3, 1)
    );
    assert!(
        !authority.clear_staged_replay_allocations(),
        "nothing was staged"
    );
    assert_eq!(authority.frontier_snapshot(), vec![(E, P, 3)]);
}

#[test]
fn a_live_slot_outside_the_staged_block_is_refused() {
    let authority = RecordIdentitySubsystem::default();
    authority
        .stage_replay_allocations(vec![live(0, 0)], vec![cancelled(1)])
        .unwrap();
    let mut pending = authority.begin_allocations();
    assert_eq!(
        pending.cancelled(),
        &[cancelled(1)],
        "replay carries the cancelled evidence"
    );
    assert_eq!(pending.reserve(E, P).unwrap().slot, 0);
    let generations = authority.generation_snapshot();
    assert_eq!(
        generations,
        vec![(E, P, 0, 1), (E, P, 1, 1)],
        "the cancelled slot is burned"
    );
    drop(pending);
    let fresh = RecordIdentitySubsystem::default();
    fresh
        .stage_replay_allocations(vec![live(0, 1)], vec![cancelled(0)])
        .unwrap();
    fresh.clear_staged_replay_allocations();
    let staged = StagedReplayAllocations {
        live: vec![live(0, 5)],
        cancelled: Vec::new(),
    };
    let mut outside = PendingRecordAllocations::replaying(fresh, staged);
    assert!(matches!(
        outside.reserve(E, P).unwrap_err(),
        RecordAllocationDenial::ReplayAppendFrontierMismatch {
            observed_slot: 5,
            ..
        }
    ));
}

fn create(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: P,
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        fields: Default::default(),
    }))
}

#[test]
fn live_evidence_takes_the_merged_plan_order_and_cancels_into_held_slots() {
    let authority = RecordIdentitySubsystem::default();
    authority.restore_generation(E, P, 9, 4);
    authority.restore_reusable(E, P, 9);
    let mut pending = authority.begin_allocations();
    let mut records = Vec::new();
    for key in ["b", "a", "c"] {
        pending.begin_intent(super::allocation_source::AllocationSource::of(&create(key)));
        let slot = pending.reserve(E, P).unwrap();
        records.push(entity(slot.slot as u64, slot.generation));
        pending.record(records.last().unwrap().clone());
    }
    let [b, a, c] = [records[0].clone(), records[1].clone(), records[2].clone()];
    assert_eq!(b, entity(9, 5), "b reclaims slot 9 at its next generation");
    pending.cancel(&b).unwrap();
    pending.cancel(&c).unwrap();
    assert!(
        pending.reorder_live(&[create("b"), create("a")]).is_err(),
        "b was cancelled"
    );
    pending.reorder_live(&[create("a")]).unwrap();
    let a_live = CanonicalRecordAllocation::with_origin(
        0,
        a.clone(),
        RecordAllocationOrigin::AppendFrontier,
    );
    assert_eq!(pending.canonical(), &[a_live.clone()]);
    let mut sibling = authority.begin_allocations();
    let after = sibling.reserve(E, P).unwrap().slot;
    assert_ne!(after, 9, "a held reclaimed slot is not reissued");
    drop(sibling);
    let cancelled_evidence = pending.cancelled().to_vec();
    pending.commit();
    assert_eq!(
        authority.reusable_snapshot(),
        vec![(E, P, 9)],
        "the reclaimed slot returns"
    );
    let frontier = authority.frontier_snapshot();
    // Replay of that commit, then of a next commit reclaiming slot 9 again.
    let cold = RecordIdentitySubsystem::default();
    cold.restore_generation(E, P, 9, 4);
    cold.restore_reusable(E, P, 9);
    cold.stage_replay_allocations(vec![a_live], cancelled_evidence)
        .unwrap();
    let RecordRef::Entity(a_id) = &a else {
        unreachable!()
    };
    assert_eq!(replay(&cold, 1), vec![a_id.slot_index()]);
    assert_eq!(
        cold.reusable_snapshot(),
        vec![(E, P, 9)],
        "replay keeps the cancelled slot reusable"
    );
    let RecordRef::Entity(c_id) = &c else {
        unreachable!()
    };
    assert_eq!(frontier, vec![(E, P, after + 1)]);
    assert_eq!(
        cold.frontier_snapshot(),
        vec![(E, P, c_id.slot_index() + 1)]
    );
    cold.clear_staged_replay_allocations();
    let next = CanonicalRecordAllocation::with_origin(
        0,
        entity(9, 6),
        RecordAllocationOrigin::Reclaimed {
            prior_generation: 5,
        },
    );
    cold.stage_replay_allocations(vec![next], Vec::new())
        .unwrap();
    assert_eq!(replay(&cold, 1), vec![9]);
}
