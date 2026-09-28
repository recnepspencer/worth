use super::*;
use std::num::{NonZeroU32, NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyLimits, PhysicalResidencyPool,
    PhysicalSpeculativeWorkKind as Speculation,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

#[test]
fn allocator_index_charge_survives_claims_until_last_owner_drops() {
    let bytes = NonZeroU64::new(65_536).unwrap();
    let frames = NonZeroU32::new(4).unwrap();
    let identity = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([113; 16]).unwrap(),
    )
    .published_identity();
    let limits = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(4 * bytes.get()).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(bytes)
        .dirty_replacement_bytes(bytes)
        .frame_entries(frames)
        .pinned_frames(frames)
        .pin_leases(frames)
        .dirty_frames(frames)
        .operation_bytes(bytes)
        .scope_bytes(Scope::ForegroundRead, bytes)
        .scope_bytes(Scope::ForegroundWrite, bytes)
        .scope_bytes(Scope::Recovery, bytes)
        .scope_bytes(Scope::Scrub, bytes)
        .scope_bytes(Scope::Maintenance, bytes)
        .scope_bytes(Scope::Verification, bytes)
        .scope_bytes(Scope::Blob, bytes)
        .speculative_frames(Speculation::Prefetch, frames)
        .speculative_frames(Speculation::ReadAhead, frames)
        .speculative_frames(Speculation::WriteBehind, frames)
        .admit(NonZeroU64::MIN)
        .unwrap();
    let pool = PhysicalResidencyPool::open(identity, limits).unwrap();
    let mut allocator = owner(2);
    allocator.retain_capacity_charge(pool.begin_foreground_write_operation(bytes).unwrap());
    allocator.restore_free_range(range(1, 0, 8192)).unwrap();
    let shared = Arc::new(Mutex::new(allocator));
    let claim = ArenaReservation::reserve(&shared, 4096).unwrap();
    assert_eq!(pool.counters().active_operation_bytes(), bytes.get());
    assert!(
        pool.begin_foreground_write_operation(NonZeroU64::MIN)
            .is_err(),
        "index capacity cannot also fund candidate frame work"
    );
    drop(shared);
    assert_eq!(
        pool.counters().active_operation_bytes(),
        bytes.get(),
        "the reservation still owns the allocator indexes"
    );
    drop(claim);
    assert_eq!(pool.counters().active_operation_bytes(), 0);
    assert!(!pool.close().requires_inspection());
}
