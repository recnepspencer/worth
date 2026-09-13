use std::num::{NonZeroU32, NonZeroU64};

use worth_store_buffer_pool::{
    PhysicalFrameAccess, PhysicalFrameKey, PhysicalOperationAllocationScope,
    PhysicalResidencyLimits, PhysicalResidencyPool, PhysicalSpeculativeWorkKind,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, FreeSpaceBlockReference, FreeSpaceKey, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, RecordAllocationClass, RecordArtifactFile,
    RecordFrameCoordinate,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use crate::physical_runtime::lifecycle::LifecycleCoordinator;

pub(super) fn loaded_manifest(
    store: StableStoreIdentity,
    generation: u64,
    bytes: &[u8],
) -> (
    PhysicalResidencyPool,
    worth_store_buffer_pool::OperationAllocationGrant,
    worth_store_buffer_pool::PhysicalFrameLease,
) {
    loaded_frame(store, manifest_coordinate(generation, bytes.len()), bytes)
}

pub(super) fn loaded_frame(
    store: StableStoreIdentity,
    coordinate: RecordFrameCoordinate,
    bytes: &[u8],
) -> (
    PhysicalResidencyPool,
    worth_store_buffer_pool::OperationAllocationGrant,
    worth_store_buffer_pool::PhysicalFrameLease,
) {
    let pool = PhysicalResidencyPool::open(store, residency_limits(bytes.len() as u32)).unwrap();
    let allocation = pool
        .begin_operation(
            PhysicalOperationAllocationScope::ForegroundRead,
            NonZeroU64::new(bytes.len() as u64).unwrap(),
        )
        .unwrap();
    let key = PhysicalFrameKey::new(store, coordinate);
    let PhysicalFrameAccess::Fault(fault) = pool.access_frame(&allocation, key).unwrap() else {
        panic!("fresh pool must yield a frame fault");
    };
    let lease = fault
        .load(|target| {
            target.copy_from_slice(bytes);
            Ok::<(), ()>(())
        })
        .unwrap();
    (pool, allocation, lease)
}

pub(super) fn manifest_coordinate(generation: u64, length: usize) -> RecordFrameCoordinate {
    RecordFrameCoordinate::new(
        RecordArtifactFile::RootManifest { generation },
        0,
        length as u32,
    )
    .unwrap()
}

fn residency_limits(frame_bytes: u32) -> PhysicalResidencyLimits {
    let bytes = NonZeroU64::new(u64::from(frame_bytes)).unwrap();
    let count = NonZeroU32::new(2).unwrap();
    let mut builder = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(u64::from(frame_bytes) * 3 + 4096).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(NonZeroU64::new(4096).unwrap())
        .frame_entries(count)
        .pinned_frames(count)
        .pin_leases(count)
        .dirty_frames(count)
        .dirty_replacement_bytes(bytes)
        .operation_bytes(bytes);
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Scrub,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        builder = builder.scope_bytes(scope, bytes);
    }
    for kind in [
        PhysicalSpeculativeWorkKind::Prefetch,
        PhysicalSpeculativeWorkKind::ReadAhead,
        PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, count);
    }
    builder.admit(NonZeroU64::MIN).unwrap()
}

pub(super) fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(super) fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB16)
        .admit()
        .unwrap()
}

pub(super) fn manifest_bytes(generation: u64, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free_space_root = FreeSpaceBlockReference::new(generation, 1, 0, 41, key, key).unwrap();
    DurablePhysicalRootManifest::builder(generation, 71, 2, 43)
        .free_space_root(Some(free_space_root))
        .admit()
        .unwrap()
        .encode(format)
}

pub(super) fn lifecycle() -> LifecycleCoordinator {
    let lifecycle = LifecycleCoordinator::admitted();
    lifecycle.progress_to_media_owned();
    lifecycle
}

pub(super) fn scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
    length: usize,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        PhysicalByteRange::new(0, length as u64).unwrap(),
    )
    .unwrap()
}
