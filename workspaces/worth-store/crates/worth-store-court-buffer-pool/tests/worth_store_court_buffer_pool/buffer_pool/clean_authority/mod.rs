#[allow(unused_imports)] use crate::clean_authority::*;
use {worth_store_buffer_pool::PhysicalFrameKey, worth_store_buffer_pool::PhysicalOperationAllocationScope, worth_store_buffer_pool::PhysicalResidencyDenial, worth_store_buffer_pool::PhysicalResidencyLimits, worth_store_buffer_pool::PhysicalResidencyPool, worth_store_buffer_pool::PhysicalResidencyPoolOwner, worth_store_buffer_pool::PhysicalSpeculativeWorkKind};
use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

#[test]
fn owner_capabilities_clean_candidate_and_writeback_transitions() {
    let (pool, candidate_clean, writeback_clean) = owned_pool(41);
    let allocation = foreground_write(&pool);

    let candidate_key = frame_key(pool.store_identity(), 1);
    let candidate = pool
        .materialize_dirty_candidate(&allocation, candidate_key, |bytes| bytes.fill(0x41))
        .unwrap();
    let clean_candidate = candidate
        .complete_candidate_publication(&candidate_clean)
        .unwrap();
    assert_eq!(pool.counters().dirty_frames(), 0);
    drop(clean_candidate);

    let writeback_key = frame_key(pool.store_identity(), 2);
    let dirty = pool
        .materialize_dirty_candidate(&allocation, writeback_key, |bytes| bytes.fill(0x42))
        .unwrap();
    writeback_claim(&pool, writeback_key)
        .unwrap()
        .complete_writeback(&writeback_clean)
        .unwrap();
    assert_eq!(pool.counters().dirty_frames(), 0);
    assert_eq!(pool.counters().active_writeback_claims(), 0);

    drop(dirty);
    drop(allocation);
    assert!(!pool.close().requires_inspection());
}

#[test]
fn foreign_candidate_authority_cannot_clean_an_identical_store_incarnation() {
    let (pool, _, _) = owned_pool(42);
    let (_, foreign_candidate_clean, _) = owned_pool(42);
    let allocation = foreground_write(&pool);
    let key = frame_key(pool.store_identity(), 1);
    let dirty = pool
        .materialize_dirty_candidate(&allocation, key, |bytes| bytes.fill(0x51))
        .unwrap();

    assert!(matches!(
        dirty.complete_candidate_publication(&foreign_candidate_clean),
        Err(worth_store_buffer_pool::PhysicalResidencyDenial::CandidateCleanAuthorityMismatch)
    ));
    let counters = pool.counters();
    assert_eq!(counters.dirty_frames(), 1);
    assert_eq!(counters.candidate_frames(), 1);
    assert_eq!(counters.pinned_frames(), 0);
    assert_eq!(counters.pin_leases(), 0);

    drop(allocation);
    assert!(pool.close().requires_inspection());
}

#[test]
fn foreign_writeback_authority_releases_the_claim_without_cleaning() {
    let (pool, _, _) = owned_pool(43);
    let (_, _, foreign_writeback_clean) = owned_pool(43);
    let allocation = foreground_write(&pool);
    let key = frame_key(pool.store_identity(), 1);
    let dirty = pool
        .materialize_dirty_candidate(&allocation, key, |bytes| bytes.fill(0x61))
        .unwrap();
    let claim = writeback_claim(&pool, key).unwrap();

    assert_eq!(
        claim.complete_writeback(&foreign_writeback_clean),
        Err(worth_store_buffer_pool::PhysicalResidencyDenial::WritebackCleanAuthorityMismatch)
    );
    let counters = pool.counters();
    assert_eq!(counters.dirty_frames(), 1);
    assert_eq!(counters.active_writeback_claims(), 0);

    drop(dirty);
    drop(allocation);
    assert!(pool.close().requires_inspection());
}
