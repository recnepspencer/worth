use std::num::{NonZeroU32, NonZeroU64};

use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope, PhysicalResidencyLimits, PhysicalSpeculativeWorkKind,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

use crate::physical_runtime::{
    record_serving::RecordFramePorts, LifecycleGeneration, RuntimeIdentity,
};

pub(super) fn ports(
    operation_bytes: u64,
) -> (
    RecordFramePorts,
    worth_store_physical_format::store_namespace::StableStoreIdentity,
    RuntimeIdentity,
    LifecycleGeneration,
) {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([141; 16]).unwrap(),
    )
    .published_identity();
    let bytes = NonZeroU64::new(4096).unwrap();
    let operation = NonZeroU64::new(operation_bytes).unwrap();
    let frames = NonZeroU32::new(4).unwrap();
    let mut limits = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(12_288 + operation_bytes).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(bytes)
        .frame_entries(frames)
        .pinned_frames(frames)
        .pin_leases(frames)
        .dirty_frames(frames)
        .dirty_replacement_bytes(bytes)
        .operation_bytes(operation);
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Scrub,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        limits = limits.scope_bytes(scope, operation);
    }
    let limits = limits
        .speculative_frames(PhysicalSpeculativeWorkKind::Prefetch, frames)
        .speculative_frames(PhysicalSpeculativeWorkKind::ReadAhead, frames)
        .speculative_frames(PhysicalSpeculativeWorkKind::WriteBehind, frames)
        .admit(NonZeroU64::MIN)
        .unwrap();
    (
        RecordFramePorts::bounded(store, limits).unwrap(),
        store,
        RuntimeIdentity::from_reopened(NonZeroU64::new(11).unwrap()),
        LifecycleGeneration::from_reopened(NonZeroU64::new(3).unwrap()),
    )
}
