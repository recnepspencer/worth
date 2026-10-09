use super::*;
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange, PhysicalTierClass};

mod release_budget;
mod retained_charge;

fn range(arena: u64, offset: u64, length: u64) -> ExtentArenaRange {
    ExtentArenaRange::new(ExtentArenaId::new(arena).unwrap(), offset, length).unwrap()
}

fn owner(next: u64) -> ExtentArenaAllocationOwner {
    ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 64, next).unwrap()
}

#[test]
fn tiered_allocator_segregates_best_fit_and_skips_at_most_two_arena_names() {
    let mut allocator =
        ExtentArenaAllocationOwner::new_tiered(ExtentArenaCapacity::DEFAULT, 4096, 64, 7, Some(7))
            .unwrap();
    let hot = allocator
        .reserve_in_tier(4096, PhysicalTierClass::Hot)
        .unwrap()
        .1;
    assert_eq!(hot, range(8, 0, 4096));
    let primary = allocator.reserve(4096).unwrap().1;
    assert_eq!(primary, range(7, 0, 4096));
    let cold = allocator
        .reserve_in_tier(4096, PhysicalTierClass::Cold)
        .unwrap()
        .1;
    assert_eq!(cold, range(9, 0, 4096));
    assert_ne!(primary.arena(), hot.arena());
    assert_ne!(hot.arena(), cold.arena());

    let mut cold_first =
        ExtentArenaAllocationOwner::new_tiered(ExtentArenaCapacity::DEFAULT, 4096, 64, 7, Some(7))
            .unwrap();
    assert_eq!(
        cold_first
            .reserve_in_tier(4096, PhysicalTierClass::Cold)
            .unwrap()
            .1,
        range(9, 0, 4096)
    );
    assert_eq!(cold_first.reserve(4096).unwrap().1, range(7, 0, 4096));
    assert_eq!(
        cold_first
            .reserve_in_tier(4096, PhysicalTierClass::Hot)
            .unwrap()
            .1,
        range(8, 0, 4096)
    );
}

#[test]
fn legacy_allocator_denies_non_primary_before_any_claim() {
    let mut allocator = owner(7);
    assert_eq!(
        allocator.reserve_in_tier(4096, PhysicalTierClass::Cold),
        Err(ArenaAllocationDenial::InvalidGeometry)
    );
    assert_eq!(allocator.reserve(4096).unwrap().1, range(7, 0, 4096));
}

#[test]
fn denied_tier_skip_does_not_consume_frontier_or_claim_budget() {
    let mut allocator =
        ExtentArenaAllocationOwner::new_tiered(ExtentArenaCapacity::DEFAULT, 4096, 3, 7, Some(7))
            .unwrap();
    assert_eq!(
        allocator.reserve_in_tier(4096, PhysicalTierClass::Cold),
        Err(ArenaAllocationDenial::RangeBudget {
            required: 4,
            maximum: 3,
        })
    );
    assert_eq!(allocator.reserve(4096).unwrap().1, range(7, 0, 4096));
}

#[test]
fn epoch_fence_atomically_denies_claims_and_quarantines_uncertain_effects() {
    let shared = Arc::new(Mutex::new(owner(7)));
    let fence = ArenaTierEpochFence::begin(&shared, 7).unwrap();
    let concurrent = Arc::clone(&shared);
    let denial = std::thread::spawn(move || ArenaReservation::reserve(&concurrent, 4096).err())
        .join()
        .unwrap();
    assert_eq!(denial, Some(ArenaAllocationDenial::EvacuationBusy));
    drop(fence);
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        range(7, 0, 4096)
    );

    let shared = Arc::new(Mutex::new(owner(7)));
    let mut uncertain = ArenaTierEpochFence::begin(&shared, 7).unwrap();
    uncertain.mark_effect_may_exist();
    drop(uncertain);
    assert!(matches!(
        ArenaReservation::reserve(&shared, 4096),
        Err(ArenaAllocationDenial::EvacuationBusy)
    ));
}

#[test]
fn completed_epoch_fence_preserves_old_primary_and_admits_new_hot_arena() {
    let shared = Arc::new(Mutex::new(owner(7)));
    let mut fence = ArenaTierEpochFence::begin(&shared, 7).unwrap();
    assert_eq!(fence.epoch(), 7);
    fence.mark_effect_may_exist();
    fence.complete().unwrap();
    assert!(matches!(
        ArenaTierEpochFence::begin(&shared, 7),
        Err(ArenaAllocationDenial::EvacuationBusy)
    ));
    let hot = ArenaReservation::reserve_in_tier(&shared, 4096, PhysicalTierClass::Hot).unwrap();
    assert_eq!(hot.range(), range(8, 0, 4096));
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        range(7, 0, 4096)
    );
}

#[test]
fn full_highest_arena_does_not_reset_reopen_frontier() {
    let mut allocator = owner(7);
    let (_, allocated) = allocator.reserve(4096).unwrap();
    assert_eq!(allocated, range(7, 0, 4096));
}

#[test]
fn best_fit_splits_and_adjacent_cancellation_coalesces() {
    let mut allocator = owner(4);
    allocator.restore_free_range(range(1, 0, 32768)).unwrap();
    allocator.restore_free_range(range(2, 4096, 8192)).unwrap();
    let (first, selected) = allocator.reserve(4096).unwrap();
    assert_eq!(selected, range(2, 4096, 4096));
    let (second, selected) = allocator.reserve(4096).unwrap();
    assert_eq!(selected, range(2, 8192, 4096));
    allocator.cancel(first).unwrap();
    allocator.cancel(second).unwrap();
    let (_, selected) = allocator.reserve(8192).unwrap();
    assert_eq!(selected, range(2, 4096, 8192));
}

#[test]
fn restore_rejects_overlap_and_future_arena_without_mutation() {
    let mut allocator = owner(3);
    allocator.restore_free_range(range(1, 0, 8192)).unwrap();
    assert_eq!(
        allocator.restore_free_range(range(1, 4096, 8192)),
        Err(ArenaAllocationDenial::Overlap)
    );
    assert_eq!(
        allocator.restore_free_range(range(3, 0, 4096)),
        Err(ArenaAllocationDenial::InvalidGeometry)
    );
    assert_eq!(allocator.reserve(8192).unwrap().1, range(1, 0, 8192));
}

#[test]
fn dropped_pre_wal_claim_reuses_but_exposed_claim_does_not() {
    let shared = Arc::new(Mutex::new(owner(1)));
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_eq!(claim.range(), range(1, 0, 4096));
    drop(claim);
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_eq!(claim.range(), range(1, 0, 4096));
    claim.expose_to_wal();
    drop(claim);
    let next = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_eq!(next.range(), range(1, 4096, 4096));
    next.publish();
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        range(1, 8192, 4096)
    );
}

#[test]
fn durable_copy_obligation_retains_dropped_carrier_until_resolved() {
    let shared = Arc::new(Mutex::new(owner(1)));
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    let original = claim.range();
    let obligation = claim.retain_obligation();
    claim.expose_to_wal();
    drop(claim);
    let other = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_ne!(other.range(), original);
    obligation.cancel_after_resolution().unwrap();
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        original
    );
    assert!(
        obligation.cancel_after_resolution().is_err(),
        "resolution is single-use"
    );
}

#[test]
fn published_copy_obligation_cannot_cancel_live_destination() {
    let shared = Arc::new(Mutex::new(owner(1)));
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    let original = claim.range();
    let obligation = claim.retain_obligation();
    claim.expose_to_wal();
    claim.publish();
    assert!(obligation.cancel_after_resolution().is_err());
    drop(obligation);
    assert_ne!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        original
    );
}

#[test]
fn best_fit_split_uses_one_slot_and_denial_preserves_claim_and_frontier() {
    let mut allocator =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 3, 1).unwrap();
    let (first_token, first) = allocator.reserve(4096).unwrap();
    let (second_token, second) = allocator.reserve(4096).unwrap();
    assert_eq!(second, range(first.arena().get(), first.end(), 4096));
    assert_eq!(
        allocator.reserve(4096),
        Err(ArenaAllocationDenial::RangeBudget {
            required: 4,
            maximum: 3,
        })
    );
    allocator.cancel(second_token).unwrap();
    let (retry_token, retry) = allocator.reserve(4096).unwrap();
    assert_eq!(retry_token, second_token + 1);
    assert_eq!(retry, second);
    allocator.cancel(first_token).unwrap();
    allocator.cancel(retry_token).unwrap();
    let capacity = ExtentArenaCapacity::DEFAULT.get();
    assert_eq!(
        allocator.reserve(capacity).unwrap().1,
        range(1, 0, capacity)
    );
    assert_eq!(
        allocator.reserve(capacity).unwrap().1,
        range(2, 0, capacity)
    );
}

#[test]
fn best_fit_exact_match_needs_no_spare_slot_at_full_quota() {
    let mut allocator =
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, 2, 2).unwrap();
    allocator.restore_free_range(range(1, 0, 4096)).unwrap();
    allocator.restore_free_range(range(1, 8192, 4096)).unwrap();
    let (token, selected) = allocator.reserve(4096).unwrap();
    assert_eq!(selected, range(1, 0, 4096));
    allocator.cancel(token).unwrap();
    assert_eq!(allocator.reserve(4096).unwrap().1, selected);
}

#[test]
fn evacuation_exclusion_fences_allocations_and_new_releases_until_cancelled() {
    let mut allocator = owner(3);
    allocator.restore_free_range(range(1, 0, 8192)).unwrap();
    allocator.restore_free_range(range(2, 0, 16384)).unwrap();
    let shared = Arc::new(Mutex::new(allocator));
    let exclusion = ArenaEvacuationLease::acquire(&shared, ExtentArenaId::new(1).unwrap()).unwrap();
    shared
        .lock()
        .unwrap()
        .admit_durable_release(range(1, 8192, 4096))
        .unwrap();
    let destination = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_eq!(destination.range().arena().get(), 2);
    drop(destination);
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096)
            .unwrap()
            .range()
            .arena()
            .get(),
        2
    );
    drop(exclusion);
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096).unwrap().range(),
        range(1, 0, 4096)
    );
}

#[test]
fn in_flight_source_claim_blocks_evacuation_and_forgotten_ids_are_not_recycled() {
    let shared = Arc::new(Mutex::new(owner(1)));
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    let arena = claim.range().arena();
    assert!(matches!(
        ArenaEvacuationLease::acquire(&shared, arena),
        Err(ArenaAllocationDenial::EvacuationBusy)
    ));
    drop(claim);
    ArenaEvacuationLease::acquire(&shared, arena)
        .unwrap()
        .forget_after_publication();
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096)
            .unwrap()
            .range()
            .arena()
            .get(),
        2
    );
}

#[test]
fn exposed_evacuation_abandonment_keeps_source_excluded() {
    let mut allocator = owner(3);
    allocator.restore_free_range(range(1, 0, 8192)).unwrap();
    allocator.restore_free_range(range(2, 0, 16384)).unwrap();
    let shared = Arc::new(Mutex::new(allocator));
    let mut lease = ArenaEvacuationLease::acquire(&shared, ExtentArenaId::new(1).unwrap()).unwrap();
    lease.expose_to_retirement();
    drop(lease);
    assert_eq!(
        ArenaReservation::reserve(&shared, 4096)
            .unwrap()
            .range()
            .arena()
            .get(),
        2
    );
}
