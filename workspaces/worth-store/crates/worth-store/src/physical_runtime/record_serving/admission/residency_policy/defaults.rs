use std::num::{NonZeroU32, NonZeroU64};

use super::{
    AdmittedPhysicalRecordResidencyPolicy, PhysicalOperationAllocationScope,
    PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind,
};
use crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat;

const CANONICAL_FRAME_METADATA_BYTES: u64 = 3 * 1024 * 1024;

pub(in crate::physical_runtime::record_serving::admission) fn canonical_residency_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(384 * 1024 * 1024))
        .resident_bytes(bytes(64 * 1024 * 1024))
        // C.9 retains one exact-scope validation record in every C.6 frame
        // entry, so the declared metadata envelope includes that fixed cost.
        .metadata_bytes(bytes(CANONICAL_FRAME_METADATA_BYTES))
        .frame_entries(count(4096))
        .pinned_frames(count(256))
        .pin_leases(count(512))
        .dirty_frames(count(64))
        .dirty_replacement_bytes(bytes(64 * 1024 * 1024))
        .operation_bytes(bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::ForegroundRead, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::ForegroundWrite, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Recovery, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Scrub, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Maintenance, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Verification, bytes(256 * 1024 * 1024))
        .scope_bytes(Scope::Blob, bytes(256 * 1024 * 1024))
        .speculative_frames(Speculation::Prefetch, count(256))
        .speculative_frames(Speculation::ReadAhead, count(256))
        .speculative_frames(Speculation::WriteBehind, count(64))
        .admit(format)
        .into_result()
        .expect("canonical residency declaration must admit every supported format")
}

const fn bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("canonical residency byte limits are nonzero")
}

const fn count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("canonical residency count limits are nonzero")
}
