mod free_reuse_ordering;

use worth_store_test_support::harness::physical_isolation::epoch_scope as support;
use worth_store_test_support::harness::physical_isolation::reclaim as reclaim_support;

use {
    worth_store_physical_isolation::reject_backend_residue_as_reclaim_authority,
    worth_store_physical_isolation::reject_copied_read_plan_fields_as_reclaim_authority,
    worth_store_physical_isolation::reject_current_root_absence_as_reclaim_authority,
    worth_store_physical_isolation::reject_raw_reader_handle_scan_as_reclaim_authority,
    worth_store_physical_isolation::AllocatorPublicationReceipt,
    worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot,
    worth_store_physical_isolation::CrashStableReclaimReuseFence,
    worth_store_physical_isolation::ExecutedReachabilityEvidence,
    worth_store_physical_isolation::FreeReuseFenceDenial,
    worth_store_physical_isolation::GenerationAdvanceReceipt,
    worth_store_physical_isolation::HazardLeaseDenial,
    worth_store_physical_isolation::HazardLeaseKind,
    worth_store_physical_isolation::HazardLeaseTable,
    worth_store_physical_isolation::HazardLeaseTableCapacity,
    worth_store_physical_isolation::LeaseExpiryPosture,
    worth_store_physical_isolation::PhysicalOrderingContract,
    worth_store_physical_isolation::PhysicalOrderingSite,
    worth_store_physical_isolation::ReclaimDenial,
    worth_store_physical_isolation::ReclaimEligibilityProof,
};
use {
    worth_store_test_support::harness::physical_isolation::epoch_scope::current_root_from_authority,
    worth_store_test_support::harness::physical_isolation::epoch_scope::physical_authority_from_operation_digest_closeout,
};
use {
    worth_store_test_support::harness::physical_isolation::reclaim::assert_reclaim_blocked_by_live_hazard,
    worth_store_test_support::harness::physical_isolation::reclaim::assert_reclaim_eligible_from_live_table,
    worth_store_test_support::harness::physical_isolation::reclaim::eligibility_after_releases,
    worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture,
};

#[test]
fn live_hazard_lease_blocks_reclaim_until_release() {
    let world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(7);
    let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
        worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(4).unwrap(),
    );
    let active = table.acquire(world.root, world.lease.clone()).unwrap();
    let blocked = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        table.live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();

    assert!(matches!(
        blocked.try_reclaim(),
        Err(
            worth_store_physical_isolation::ReclaimDenial::BlockedByLiveHazardLease {
                kind: worth_store_physical_isolation::HazardLeaseKind::ForegroundRead,
                overlapping_ranges: 1,
                ..
            }
        )
    ));
    assert_eq!(blocked.counters().blocked_reclaims(), 1);

    let release = table.release(active).unwrap();
    let eligible = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        table.live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();

    assert!(eligible.decision().is_eligible());
    assert_eq!(
        eligible
            .admit_reachability_removal()
            .unwrap()
            .counters()
            .eligible_reclaims(),
        1
    );
    assert_eq!(
        release.footprint_basis(),
        world.candidates.footprint_basis()
    );
}

#[test]
fn expiry_alone_is_not_reclaim_authority() {
    let released_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(11);
    let revoked_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(12);
    let owned_copy_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(13);
    let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
        worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(3).unwrap(),
    );
    let released_active = table
        .acquire(released_world.root, released_world.lease.clone())
        .unwrap();
    let revoked_active = table
        .acquire(revoked_world.root, revoked_world.lease.clone())
        .unwrap();
    let owned_copy_active = table
        .acquire(owned_copy_world.root, owned_copy_world.lease.clone())
        .unwrap();

    let expired = worth_store_physical_isolation::LeaseExpiryPosture::expired_without_authority(
        released_active.slot(),
        released_active.generation(),
    );
    assert_eq!(
        expired.require_reclaim_authority(),
        Err(
            worth_store_physical_isolation::HazardLeaseDenial::ExpiredLeaseWithoutReleaseRevocationOrOwnedCopy {
                slot: released_active.slot()
            }
        )
    );

    let release = table.release(released_active).unwrap();
    let released = worth_store_physical_isolation::LeaseExpiryPosture::from_release(release);
    assert_eq!(released.require_reclaim_authority(), Ok(()));
    assert_reclaim_eligible_from_live_table(&released_world, &table, 2);
    assert_reclaim_blocked_by_live_hazard(&revoked_world, &table, 2);
    assert_reclaim_blocked_by_live_hazard(&owned_copy_world, &table, 2);

    let revocation = table.revoke(revoked_active).unwrap();
    let revoked = worth_store_physical_isolation::LeaseExpiryPosture::from_revocation(revocation);
    assert_eq!(revoked.require_reclaim_authority(), Ok(()));
    assert_reclaim_eligible_from_live_table(&released_world, &table, 1);
    assert_reclaim_eligible_from_live_table(&revoked_world, &table, 1);
    assert_reclaim_blocked_by_live_hazard(&owned_copy_world, &table, 1);

    let owned_copy = table.convert_to_owned_copy(owned_copy_active).unwrap();
    let copied = worth_store_physical_isolation::LeaseExpiryPosture::from_owned_copy(owned_copy);
    assert_eq!(copied.require_reclaim_authority(), Ok(()));
    assert_eq!(table.counters().released_leases(), 1);
    assert_eq!(table.counters().revoked_leases(), 1);
    assert_eq!(table.counters().owned_copy_conversions(), 1);
    assert_reclaim_eligible_from_live_table(&released_world, &table, 0);
    assert_reclaim_eligible_from_live_table(&revoked_world, &table, 0);
    assert_reclaim_eligible_from_live_table(&owned_copy_world, &table, 0);
}

#[test]
fn stale_release_is_generation_counted_and_does_not_reopen_reclaim_authority() {
    let world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(13);
    let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
        worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
    );
    let active = table.acquire(world.root, world.lease.clone()).unwrap();

    table.release(active).unwrap();
    assert!(matches!(
        table.release(active),
        Err(worth_store_physical_isolation::HazardLeaseDenial::StaleLeaseGeneration { slot, observed, .. })
            if slot == active.slot() && observed == active.generation()
    ));
    assert_eq!(table.counters().released_leases(), 1);
    assert_eq!(table.counters().stale_release_denials(), 1);

    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        table.live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    assert!(proof.decision().is_eligible());
}

#[test]
fn reclaim_denies_non_executed_authority_shortcuts() {
    assert_eq!(
        reject_backend_residue_as_reclaim_authority(),
        worth_store_physical_isolation::ReclaimDenial::BackendResidueIsNotReachabilityAuthority
    );
    assert_eq!(
        reject_current_root_absence_as_reclaim_authority(),
        worth_store_physical_isolation::ReclaimDenial::CurrentRootAbsenceIsNotReachabilityAuthority
    );
    assert_eq!(
        reject_raw_reader_handle_scan_as_reclaim_authority(),
        worth_store_physical_isolation::ReclaimDenial::RawReaderHandleScanIsNotReachabilityAuthority
    );
    assert_eq!(
        reject_copied_read_plan_fields_as_reclaim_authority(),
        worth_store_physical_isolation::ReclaimDenial::CopiedReadPlanFieldsAreNotReachabilityAuthority
    );
}

#[test]
fn executed_reachability_denies_copied_candidate_sets() {
    let released_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(41);
    let wrong_footprint_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(42);
    assert_eq!(
        worth_store_physical_isolation::ExecutedReachabilityEvidence::from_released_old_reachability(
            released_world.released,
            wrong_footprint_world.candidates.clone(),
        )
        .map(|_| ()),
        Err(worth_store_physical_isolation::ReclaimDenial::CandidateDoesNotMatchExecutedReachability {
            executed: released_world.released.footprint_basis(),
            candidate: wrong_footprint_world.candidates.footprint_basis(),
        })
    );

    let wrong_root_world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::with_operation_digest_authority(41, "phase10-wrong-candidate-root");
    assert_eq!(
        worth_store_physical_isolation::ExecutedReachabilityEvidence::from_released_old_reachability(
            released_world.released,
            wrong_root_world.candidates.clone(),
        )
        .map(|_| ()),
        Err(
            worth_store_physical_isolation::ReclaimDenial::CandidateRootDoesNotMatchExecutedReachability {
                executed: released_world.released.release_receipt().root_epoch(),
                candidate: wrong_root_world.candidates.root_epoch(),
            }
        )
    );
}

#[test]
fn every_hazard_kind_blocks_reclaim_with_exact_overlap() {
    for kind in [
        worth_store_physical_isolation::HazardLeaseKind::ForegroundRead,
        worth_store_physical_isolation::HazardLeaseKind::ScrubWindow,
        worth_store_physical_isolation::HazardLeaseKind::RecoveryVerifier,
        worth_store_physical_isolation::HazardLeaseKind::CheckpointPreservation,
        worth_store_physical_isolation::HazardLeaseKind::QuarantineHold,
        worth_store_physical_isolation::HazardLeaseKind::FutureChunkHold,
        worth_store_physical_isolation::HazardLeaseKind::BufferPoolPin,
    ] {
        let world =
            worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(19);
        let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        );
        table.acquire(world.root, world.lease_for(kind)).unwrap();
        let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
            world.executed_reachability(),
            table.live_index_snapshot(),
            worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
        )
        .unwrap();

        assert!(matches!(
            proof.try_reclaim(),
            Err(worth_store_physical_isolation::ReclaimDenial::BlockedByLiveHazardLease {
                kind: blocked,
                overlapping_ranges: 1,
                ..
            }) if blocked == kind
        ));
        assert_eq!(proof.counters().live_hazard_entries(), 1);
        assert_eq!(proof.counters().indexed_epoch_buckets_touched(), 1);
        assert_eq!(proof.counters().indexed_range_buckets_touched(), 1);
        assert_eq!(proof.counters().indexed_hazard_entries_touched(), 1);
        assert_eq!(proof.counters().overlapping_ranges(), 1);
    }
}

#[test]
fn nonmatching_epoch_hazard_range_is_not_scanned_or_blocking_authority() {
    let world =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(31);
    let other_authority = physical_authority_from_operation_digest_closeout("phase10-other-root");
    let other_root = current_root_from_authority(&other_authority);
    let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
        worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
    );
    table.acquire(other_root, world.lease.clone()).unwrap();
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        table.live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();

    assert!(proof.decision().is_eligible());
    assert_eq!(proof.counters().live_hazard_entries(), 1);
    assert_eq!(proof.counters().indexed_epoch_buckets_touched(), 0);
    assert_eq!(proof.counters().indexed_range_buckets_touched(), 0);
    assert_eq!(proof.counters().indexed_hazard_entries_touched(), 0);
    assert_eq!(proof.counters().hazard_lookup_ranges(), 0);
    assert_eq!(proof.counters().range_comparisons(), 0);
}

#[test]
fn same_epoch_nonoverlapping_hazard_range_is_indexed_out_before_entry_scan() {
    let candidate =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(37);
    let unrelated = [
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(38),
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(39),
    ];
    let mut table = worth_store_physical_isolation::HazardLeaseTable::with_capacity(
        worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(3).unwrap(),
    );
    for world in &unrelated {
        table.acquire(candidate.root, world.lease.clone()).unwrap();
    }
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        candidate.executed_reachability(),
        table.live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();

    assert!(proof.decision().is_eligible());
    assert_eq!(proof.counters().live_hazard_entries(), 2);
    assert_eq!(proof.counters().indexed_epoch_buckets_touched(), 1);
    assert_eq!(proof.counters().indexed_range_buckets_touched(), 2);
    assert_eq!(proof.counters().indexed_hazard_entries_touched(), 0);
}

#[test]
fn release_order_does_not_change_reclaim_eligibility() {
    let first =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(23);
    let second =
        worth_store_test_support::harness::physical_isolation::reclaim::ReclaimFixture::new(29);
    let forward = eligibility_after_releases([first.clone(), second.clone()]);
    let reverse = eligibility_after_releases([second, first]);

    assert_eq!(forward, reverse);
}

#[test]
fn free_reuse_requires_eligible_reclaim_generation_advance_and_allocator_publication() {
    free_reuse_ordering::generation_and_allocator_publication_precede_reuse();
}
