use super::*;

#[test]
fn clean_range_invalidation_is_atomic_scoped_and_forces_fresh_load() {
    let identity = store(114);
    let pool = PhysicalResidencyPool::open(identity, limits(256, 4, 2, 256, 8)).unwrap();
    let allocation = allocation(&pool, READ_SCOPE);
    let artifact = RecordArtifactFile::ExtentArena { arena: 9 };
    let key = |offset| {
        PhysicalFrameKey::new(
            identity,
            RecordFrameCoordinate::new(artifact, offset, 32).unwrap(),
        )
    };
    for (offset, value) in [(0, 1), (32, 2), (64, 3), (96, 4)] {
        drop(
            expect_fault(&pool, &allocation, key(offset))
                .load(|bytes| fill(bytes, value))
                .unwrap(),
        );
    }
    let pinned = expect_hit(&pool, &allocation, key(64));
    let before = pool.counters();
    assert_eq!(
        pool.invalidate_clean_range(artifact, 32, 64),
        Err(PhysicalResidencyDenial::FramePinned)
    );
    assert_eq!(pool.counters().frame_entries(), before.frame_entries());
    assert_eq!(&*expect_hit(&pool, &allocation, key(32)), &[2; 32]);
    drop(pinned);
    pool.invalidate_clean_range(artifact, 32, 64).unwrap();
    assert_eq!(pool.counters().frame_entries(), 2);
    assert_eq!(&*expect_hit(&pool, &allocation, key(0)), &[1; 32]);
    assert_eq!(&*expect_hit(&pool, &allocation, key(96)), &[4; 32]);
    let reloaded = expect_fault(&pool, &allocation, key(32))
        .load(|bytes| fill(bytes, 9))
        .unwrap();
    assert_eq!(&*reloaded, &[9; 32]);
    assert!(reloaded.integrity_validation().is_none());
}

#[test]
fn invalidation_denies_dirty_and_loading_ranges_without_weakening_claims() {
    let identity = store(115);
    let pool =
        PhysicalResidencyPool::open(identity, limits(256, 4, 2, candidate_batch_bytes(1), 8))
            .unwrap();
    let allocation = candidate_allocation(&pool, 1);
    let artifact = RecordArtifactFile::ExtentArena { arena: 10 };
    let key = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(artifact, 16, 32).unwrap(),
    );
    let loading = expect_fault(&pool, &allocation, key);
    assert_eq!(
        pool.invalidate_clean_range(artifact, 0, 64),
        Err(PhysicalResidencyDenial::FrameIdentityOccupied)
    );
    drop(loading.load(|bytes| fill(bytes, 3)).unwrap());
    let dirty_key = PhysicalFrameKey::new(
        identity,
        RecordFrameCoordinate::new(artifact, 64, 32).unwrap(),
    );
    let dirty = pool
        .materialize_dirty_candidate(&allocation, dirty_key, |bytes| bytes.fill(4))
        .unwrap();
    drop(dirty);
    assert_eq!(
        pool.invalidate_clean_range(artifact, 0, 96),
        Err(PhysicalResidencyDenial::FrameDirty)
    );
    assert_eq!(&*expect_hit(&pool, &allocation, key), &[3; 32]);
    pool.inner.publish_clean(dirty_key).unwrap();
}
