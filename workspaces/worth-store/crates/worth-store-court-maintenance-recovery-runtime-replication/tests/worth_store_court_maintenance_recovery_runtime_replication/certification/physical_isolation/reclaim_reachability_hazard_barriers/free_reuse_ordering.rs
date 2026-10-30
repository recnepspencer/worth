use worth_store_test_support::harness::physical_isolation::epoch_scope::current_generation_page_reference;

pub(super) fn generation_and_allocator_publication_precede_reuse() {
    let world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(17);
    let (blocked_proof, blocked_active) = {
        let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        );
        let active = table.acquire(world.root, world.lease.clone()).unwrap();
        let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
            world.executed_reachability(),
            table.live_index_snapshot(),
            worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
        )
        .unwrap();
        (proof, active)
    };
    let old_identity = current_generation_page_reference(17);
    let reused_identity = current_generation_page_reference(18);
    let generation = worth_store_physical_isolation::GenerationAdvanceReceipt::from_identity_reuse(
        old_identity,
        reused_identity,
        worth_store_physical_isolation::PhysicalOrderingContract::acquire_release_for(
            worth_store_physical_isolation::PhysicalOrderingSite::GenerationAdvancement,
        ),
    )
    .unwrap();
    let allocator = worth_store_physical_isolation::AllocatorPublicationReceipt::from_ordering(
        worth_store_physical_isolation::PhysicalOrderingContract::acquire_release_for(
            worth_store_physical_isolation::PhysicalOrderingSite::AllocatorPublication,
        ),
    )
    .unwrap();

    assert_eq!(
        worth_store_physical_isolation::GenerationAdvanceReceipt::from_identity_reuse(
            old_identity,
            reused_identity,
            worth_store_physical_isolation::PhysicalOrderingContract::root_swap_acquire_release(),
        )
        .map(|_| ()),
        Err(worth_store_physical_isolation::FreeReuseFenceDenial::GenerationAdvancementOrderingNotCrashStable)
    );
    assert_eq!(
        worth_store_physical_isolation::AllocatorPublicationReceipt::from_ordering(
            worth_store_physical_isolation::PhysicalOrderingContract::root_swap_acquire_release()
        )
        .map(|_| ()),
        Err(worth_store_physical_isolation::FreeReuseFenceDenial::AllocatorPublicationOrderingNotCrashStable)
    );

    assert!(matches!(
        blocked_proof.admit_reachability_removal(),
        Err(worth_store_physical_isolation::ReclaimDenial::BlockedByLiveHazardLease {
            kind: worth_store_physical_isolation::HazardLeaseKind::ForegroundRead,
            overlapping_ranges: 1,
            slot,
            generation,
        }) if slot == blocked_active.slot() && generation == blocked_active.generation()
    ));

    let eligible = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        )
        .live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    let removal = eligible.admit_reachability_removal().unwrap();
    let unrelated_generation =
        worth_store_physical_isolation::GenerationAdvanceReceipt::from_identity_reuse(
            current_generation_page_reference(18),
            current_generation_page_reference(19),
            worth_store_physical_isolation::PhysicalOrderingContract::acquire_release_for(
                worth_store_physical_isolation::PhysicalOrderingSite::GenerationAdvancement,
            ),
        )
        .unwrap();
    assert_eq!(
        worth_store_physical_isolation::CrashStableReclaimReuseFence::admit_after_reclaim(
            removal.clone(),
            unrelated_generation,
            allocator
        )
        .map(|_| ()),
        Err(worth_store_physical_isolation::FreeReuseFenceDenial::ReclaimRemovalDoesNotCoverReusedIdentity)
    );
    let fence = worth_store_physical_isolation::CrashStableReclaimReuseFence::admit_after_reclaim(
        removal, generation, allocator,
    )
    .unwrap();

    assert_eq!(fence.generation_advance().old_identity(), old_identity);
    assert_eq!(
        fence.generation_advance().reused_identity(),
        reused_identity
    );
    assert_eq!(fence.reclaim_counters().eligible_reclaims(), 1);
    assert_eq!(
        fence.reachability_removal().evidence().root_epoch(),
        world.root.epoch()
    );
}
