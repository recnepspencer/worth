use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalOperationAllocationScope as Scope,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordOpen,
    PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind as Speculation,
};

#[path = "support/durability.rs"]
mod durability;

fn main() {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let policy = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(65_536))
        .resident_bytes(bytes(16_384))
        .metadata_bytes(bytes(8192))
        .frame_entries(count(8))
        .pinned_frames(count(8))
        .pin_leases(count(2))
        .dirty_frames(count(4))
        .dirty_replacement_bytes(bytes(16_384))
        .operation_bytes(bytes(16_384))
        .scope_bytes(Scope::ForegroundRead, bytes(16_384))
        .scope_bytes(Scope::ForegroundWrite, bytes(16_384))
        .scope_bytes(Scope::Recovery, bytes(16_384))
        .scope_bytes(Scope::Scrub, bytes(16_384))
        .scope_bytes(Scope::Maintenance, bytes(16_384))
        .scope_bytes(Scope::Verification, bytes(16_384))
        .scope_bytes(Scope::Blob, bytes(16_384))
        .speculative_frames(Speculation::Prefetch, count(8))
        .speculative_frames(Speculation::ReadAhead, count(8))
        .speculative_frames(Speculation::WriteBehind, count(4))
        .admit(format)
        .into_result()
        .unwrap();
    let root = tempfile::Builder::new()
        .prefix("worth-store-admitted-residency-policy-supported-")
        .tempdir()
        .unwrap();
    let media = durability::media(root.path());
    let durability = durability::admitted(&media);
    let _request =
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy);
    media.close();
}

fn bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

fn count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
