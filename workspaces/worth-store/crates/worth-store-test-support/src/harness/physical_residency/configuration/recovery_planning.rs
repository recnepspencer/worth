use worth_store::physical_runtime::{
    ManifestEntryCapacity, PageFillPercent, PhysicalOperationAllocationScope as Scope,
    PhysicalRecordPlacementPolicy, PhysicalRecordResidencyPolicy,
    PhysicalSpeculativeWorkKind as Speculation, RecordByteLimit, SegmentPageCount,
};

use super::{
    admitted_store_base, bytes, frames, PhysicalResidencyStoreConfiguration, FIXTURE_FRAME_BYTES,
    FIXTURE_METADATA_BYTES,
};

const OPERATION_BYTES: u64 = 32 * 1024 * 1024;
const TOTAL_BYTES: u64 = OPERATION_BYTES + FIXTURE_METADATA_BYTES + 4 * FIXTURE_FRAME_BYTES;

pub(in crate::harness::physical_residency) fn recovery_planning_configuration(
) -> PhysicalResidencyStoreConfiguration {
    configuration(admitted_store_base())
}

pub(in crate::harness::physical_residency) fn dense_recovery_planning_configuration(
    segment_pages: u32,
) -> PhysicalResidencyStoreConfiguration {
    compact_recovery_planning_configuration(segment_pages, 128)
}

pub(in crate::harness::physical_residency) fn compact_recovery_planning_configuration(
    segment_pages: u32,
    manifest_capacity: u16,
) -> PhysicalResidencyStoreConfiguration {
    let base = admitted_store_base();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(segment_pages).unwrap())
        .extent_threshold(RecordByteLimit::new(8_000).unwrap())
        .page_fill(PageFillPercent::new(50).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(manifest_capacity).unwrap())
        .admit(base.format())
        .unwrap();
    configuration(base.with_placement(placement))
}

fn configuration(
    base: super::PhysicalResidencyStoreAdmissionBase,
) -> PhysicalResidencyStoreConfiguration {
    let format = base.format();
    let mut residency = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(TOTAL_BYTES))
        .resident_bytes(bytes(4 * FIXTURE_FRAME_BYTES))
        .metadata_bytes(bytes(FIXTURE_METADATA_BYTES))
        .frame_entries(frames(64))
        .pinned_frames(frames(64))
        .pin_leases(frames(64))
        .dirty_frames(frames(32))
        .dirty_replacement_bytes(bytes(4 * FIXTURE_FRAME_BYTES))
        .operation_bytes(bytes(OPERATION_BYTES));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        residency = residency.scope_bytes(scope, bytes(OPERATION_BYTES));
    }
    let residency = residency
        .speculative_frames(Speculation::Prefetch, frames(8))
        .speculative_frames(Speculation::ReadAhead, frames(8))
        .speculative_frames(Speculation::WriteBehind, frames(8))
        .admit(format)
        .into_result()
        .unwrap();
    base.with_residency(residency)
}
