use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalOperationAllocationScope as Scope, PhysicalRecordResidencyPolicy,
    PhysicalSpeculativeWorkKind,
};

fn admit_successor_scopes(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let frames = |value| NonZeroU32::new(value).unwrap();
    PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(262_144))
        .resident_bytes(bytes(65_536))
        .metadata_bytes(bytes(32_768))
        .frame_entries(frames(16))
        .pinned_frames(frames(8))
        .pin_leases(frames(8))
        .dirty_frames(frames(8))
        .dirty_replacement_bytes(bytes(65_536))
        .operation_bytes(bytes(32_768))
        .scope_bytes(Scope::ForegroundRead, bytes(32_768))
        .scope_bytes(Scope::ForegroundWrite, bytes(32_768))
        .scope_bytes(Scope::Recovery, bytes(8_192))
        .scope_bytes(Scope::Scrub, bytes(8_192))
        .scope_bytes(Scope::Maintenance, bytes(8_192))
        .scope_bytes(Scope::Verification, bytes(8_192))
        .scope_bytes(Scope::Blob, bytes(8_192))
        .speculative_frames(PhysicalSpeculativeWorkKind::Prefetch, frames(4))
        .speculative_frames(PhysicalSpeculativeWorkKind::ReadAhead, frames(4))
        .speculative_frames(PhysicalSpeculativeWorkKind::WriteBehind, frames(4))
        .admit(format)
        .into_result()
        .unwrap()
}

fn main() {}
