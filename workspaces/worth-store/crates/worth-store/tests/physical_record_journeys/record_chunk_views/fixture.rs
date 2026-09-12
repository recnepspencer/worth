use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecordPlacementPolicy, PhysicalOperationAllocationScope, PhysicalRecordInitialization,
    PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind, RecordServingOwnerDisposition,
    RecordServingTerminalPosture, ServingPhysicalRuntime,
};

use super::super::{media, scenario_configuration::dense_configuration};

pub(super) const FRAME_BYTES: u64 = 16_384;
const EXTENT_METADATA_BYTES: usize = 64;
pub(super) const CHUNK_PAYLOAD_BYTES: usize =
    FRAME_BYTES as usize - super::super::durable_frame_oracle::HEADER_BYTES - EXTENT_METADATA_BYTES;
pub(super) const RESIDENT_BYTES: u64 = 2 * FRAME_BYTES;

pub(super) fn initialize(root: &Path) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    initialize_with_protection(root, Default::default())
}

pub(super) fn initialize_with_protection(
    root: &Path,
    protection: worth_store::physical_runtime::PhysicalReadProtectionPolicy,
) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    let (format, placement, access) = dense_configuration(4);
    let outcome =
        initialize_record_store!(media(root), |durability| PhysicalRecordInitialization::new(
            format, placement, access, durability
        )
        .with_residency_policy(residency_policy(format))
        .with_read_protection_policy(protection),);
    let serving = match outcome.into_raw() {
        TransitionOutcome::Success(serving) => serving,
        _ => panic!("the chunk-view fixture must admit a real physical Store"),
    };
    (serving, placement)
}

pub(super) fn residency_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Speculation;

    let operation_bytes = 16 * 1024 * 1024;
    PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero_bytes(
            operation_bytes + (2 * RESIDENT_BYTES) + 16_384,
        ))
        .resident_bytes(nonzero_bytes(RESIDENT_BYTES))
        .metadata_bytes(nonzero_bytes(16_384))
        .frame_entries(nonzero_count(2))
        .pinned_frames(nonzero_count(2))
        .pin_leases(nonzero_count(2))
        .dirty_frames(nonzero_count(1))
        .dirty_replacement_bytes(nonzero_bytes(RESIDENT_BYTES))
        .operation_bytes(nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Recovery, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Scrub, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Verification, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Blob, nonzero_bytes(operation_bytes))
        .speculative_frames(Speculation::Prefetch, nonzero_count(2))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(2))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(1))
        .admit(format)
        .into_result()
        .unwrap()
}

pub(super) fn open_with_protection(
    root: &Path,
    protection: worth_store::physical_runtime::PhysicalReadProtectionPolicy,
) -> ServingPhysicalRuntime {
    let (format, _, access) = dense_configuration(4);
    let outcome = open_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(residency_policy(format))
            .with_read_protection_policy(protection)
    });
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => serving,
        _ => panic!("the existing physical Store must reopen"),
    }
}

pub(super) fn payload(length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| ((index * 31 + 7) % 251) as u8)
        .collect()
}

pub(super) fn assert_clean_close(serving: ServingPhysicalRuntime) {
    let shutdown = serving.close();
    assert_eq!(
        shutdown.records().posture(),
        RecordServingTerminalPosture::NoInspectionRequired
    );
    assert_eq!(
        shutdown.records().owner(),
        RecordServingOwnerDisposition::Released
    );
    assert_eq!(shutdown.records().counters().read_sessions_live(), 0);
    assert_eq!(shutdown.records().counters().readers_live(), 0);
    assert!(!shutdown.residency().requires_inspection());
}

fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}
