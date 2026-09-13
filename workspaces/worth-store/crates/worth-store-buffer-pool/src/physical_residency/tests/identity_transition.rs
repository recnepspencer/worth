use super::*;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityValidationRecord,
};

fn publish_complete(
    pool: &PhysicalResidencyPool,
    allocation: &ForegroundWriteAllocationGrant,
    key: PhysicalFrameKey,
    value: u8,
) {
    let candidate = PhysicalCandidateFrameKey::complete_artifact(key).unwrap();
    let mut batch = pool
        .reserve_candidate_frames(allocation, &[candidate])
        .unwrap();
    let dirty = batch
        .reserve_next(candidate)
        .unwrap()
        .materialize(|bytes| bytes.fill(value))
        .unwrap();
    pool.inner.publish_clean(key).unwrap();
    drop(dirty);
    drop(batch);
}

fn assert_only_denial_recorded(
    before: PhysicalResidencyCounters,
    after: PhysicalResidencyCounters,
) {
    assert_eq!(after.resident_bytes(), before.resident_bytes());
    assert_eq!(after.frame_entries(), before.frame_entries());
    assert_eq!(after.pinned_frames(), before.pinned_frames());
    assert_eq!(after.pin_leases(), before.pin_leases());
    assert_eq!(after.candidate_frames(), before.candidate_frames());
    assert_eq!(
        after.active_loading_frames(),
        before.active_loading_frames()
    );
    assert_eq!(after.source_loads(), before.source_loads());
    assert_eq!(after.identity_transitions(), before.identity_transitions());
    assert_eq!(after.denials(), before.denials() + 1);
}

#[test]
fn complete_artifact_promotion_rejects_nonzero_target_without_mutation() {
    let identity = store(37);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 3, 2, candidate_batch_bytes(1), 4))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let source_coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 7 }, 0, 32)
            .unwrap();
    let source = PhysicalFrameKey::new(identity, source_coordinate);
    publish_complete(&pool, &allocation, source, 7);
    let before = pool.counters();
    let target = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 8 }, 8, 32)
            .unwrap(),
    );

    assert_eq!(
        pool.promote_clean_identity(source, target).unwrap_err(),
        PhysicalResidencyDenial::CompleteArtifactRequiresOffsetZero
    );
    assert_only_denial_recorded(before, pool.counters());
    assert_eq!(&*expect_hit(&pool, &allocation, source), &[7; 32]);
}

#[test]
fn occupied_target_artifact_denies_before_source_removal() {
    let identity = store(38);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 3, 2, candidate_batch_bytes(1), 4))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let source_coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 9 }, 0, 32)
            .unwrap();
    let source = PhysicalFrameKey::new(identity, source_coordinate);
    publish_complete(&pool, &allocation, source, 9);
    let target_artifact = RecordArtifactFile::RootManifest { generation: 10 };
    let bounded =
        PhysicalBoundedFrameKey::new(identity, target_artifact, NonZeroU32::new(64).unwrap());
    let target_owner = match pool.access_bounded_frame(&allocation, bounded).unwrap() {
        PhysicalBoundedFrameAccess::Fault(owner) => owner,
        _ => panic!("target artifact must begin one bounded load"),
    };
    let before = pool.counters();
    let target = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(target_artifact, 0, 32).unwrap(),
    );

    assert_eq!(
        pool.promote_clean_identity(source, target).unwrap_err(),
        PhysicalResidencyDenial::ArtifactIdentityOccupied
    );
    assert_only_denial_recorded(before, pool.counters());
    assert_eq!(&*expect_hit(&pool, &allocation, source), &[9; 32]);
    drop(target_owner);
}

#[test]
fn legal_complete_artifact_promotion_retargets_exact_and_bounded_identity() {
    let identity = store(39);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 3, 2, candidate_batch_bytes(1), 4))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let source_artifact = RecordArtifactFile::RootManifest { generation: 11 };
    let source_coordinate = RecordFrameCoordinate::new(source_artifact, 0, 32).unwrap();
    let source = PhysicalFrameKey::new(identity, source_coordinate);
    publish_complete(&pool, &allocation, source, 5);
    let source_lease = expect_hit(&pool, &allocation, source);
    let validation = PhysicalIntegrityValidationRecord::for_test(
        PhysicalArtifactScope::root_manifest(
            identity,
            worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
                .admit()
                .unwrap(),
            11,
            PhysicalByteRange::new(0, 32).unwrap(),
        )
        .unwrap(),
        PhysicalIntegrityValidationDigest::crc32c(41),
        PhysicalIntegrityValidationDigest::crc32c(42),
        PhysicalIntegrityValidationMechanism::Crc32cV1,
    );
    source_lease
        .commit_integrity_validation(validation)
        .unwrap();
    assert_eq!(source_lease.integrity_validation(), Some(validation));
    drop(source_lease);
    let target_artifact = RecordArtifactFile::RootManifest { generation: 12 };
    let target = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(target_artifact, 0, 32).unwrap(),
    );
    let source_loads = pool.counters().source_loads();

    pool.promote_clean_identity(source, target).unwrap();
    let bounded =
        PhysicalBoundedFrameKey::new(identity, target_artifact, NonZeroU32::new(64).unwrap());
    let hit = match pool.access_bounded_frame(&allocation, bounded).unwrap() {
        PhysicalBoundedFrameAccess::Hit(hit) => hit,
        _ => panic!("promoted complete artifact must retain its bounded alias"),
    };
    assert_eq!(&*hit, &[5; 32]);
    assert_eq!(hit.key(), target);
    assert_eq!(hit.integrity_validation(), None);
    assert_eq!(pool.counters().source_loads(), source_loads);
    assert_eq!(pool.counters().identity_transitions(), 1);
}

#[test]
fn retained_failed_target_preserves_terminal_waiter_and_accounting() {
    let identity = store(40);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 4, 2, candidate_batch_bytes(1), 4))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let source = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 13 }, 0, 32)
            .unwrap(),
    );
    publish_complete(&pool, &allocation, source, 13);
    let target = PhysicalFrameKey::new(identity, coordinate(20, 32));
    let owner = expect_fault(&pool, &allocation, target);
    let waiter = match pool.access_frame(&allocation, target).unwrap() {
        PhysicalFrameAccess::Coalesced(waiter) => waiter,
        _ => panic!("overlapping exact access must retain one loading authority"),
    };
    let terminal = owner.reject_before_source();
    let before = pool.counters();

    assert_eq!(
        pool.promote_clean_identity(source, target).unwrap_err(),
        PhysicalResidencyDenial::FrameLoadTerminated(terminal)
    );
    assert_only_denial_recorded(before, pool.counters());
    assert_eq!(waiter.wait().unwrap_err(), terminal);
    assert_eq!(&*expect_hit(&pool, &allocation, source), &[13; 32]);
}

#[test]
fn clean_invalidation_preserves_retained_failure_terminal_and_accounting() {
    let identity = store(42);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 3, 1, 64, 4)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let key = PhysicalFrameKey::new(identity, coordinate(22, 32));
    let owner = expect_fault(&pool, &allocation, key);
    let waiter = match pool.access_frame(&allocation, key).unwrap() {
        PhysicalFrameAccess::Coalesced(waiter) => waiter,
        _ => panic!("overlapping exact access must retain one loading authority"),
    };
    let terminal = owner.reject_before_source();
    let before = pool.counters();

    assert_eq!(
        pool.invalidate_clean(key).unwrap_err(),
        PhysicalResidencyDenial::FrameLoadTerminated(terminal)
    );
    assert_only_denial_recorded(before, pool.counters());
    assert_eq!(waiter.wait().unwrap_err(), terminal);
    assert!(matches!(
        pool.access_frame(&allocation, key).unwrap(),
        PhysicalFrameAccess::Fault(_)
    ));
}

#[test]
fn live_loading_target_denies_promotion_and_owner_still_completes() {
    let identity = store(41);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 4, 2, candidate_batch_bytes(1), 4))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let source = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(RecordArtifactFile::RootManifest { generation: 14 }, 0, 32)
            .unwrap(),
    );
    publish_complete(&pool, &allocation, source, 14);
    let target = PhysicalFrameKey::new(identity, coordinate(21, 32));
    let owner = expect_fault(&pool, &allocation, target);
    let before = pool.counters();

    assert_eq!(
        pool.promote_clean_identity(source, target).unwrap_err(),
        PhysicalResidencyDenial::FrameIdentityOccupied
    );
    assert_only_denial_recorded(before, pool.counters());
    let loaded = owner.load(|bytes| fill(bytes, 21)).unwrap();
    assert_eq!(&*loaded, &[21; 32]);
    assert_eq!(&*expect_hit(&pool, &allocation, source), &[14; 32]);
}
