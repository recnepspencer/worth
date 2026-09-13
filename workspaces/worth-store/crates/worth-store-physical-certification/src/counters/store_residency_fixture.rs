use std::num::{NonZeroU32, NonZeroU64};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalDurabilityPolicy, AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy,
    AdmittedRecordPlacementPolicy, CheckpointMemoryLimit, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, ManifestEntryCapacity, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordInitialization, PhysicalRecordPlacementPolicy, PhysicalRecordResidencyPolicy,
    PhysicalResidencyObservation, PhysicalRuntimeAdmission, PhysicalSpeculativeWorkKind,
    PhysicalStore, PhysicalWalPolicy, RetainedWalTailLimit, ServingPhysicalRuntime,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::FilesystemAccessPosture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CertificationResidencyWorkload {
    Maintenance,
}

pub(crate) fn observe_real_store_residency(
    label: &str,
    workload: CertificationResidencyWorkload,
    allocation_bytes: u64,
) -> PhysicalResidencyObservation {
    let allocation_bytes =
        NonZeroU64::new(allocation_bytes).expect("certification allocation is non-zero");
    let root = tempfile::Builder::new()
        .prefix(label)
        .tempdir()
        .expect("certification Store directory should be created");
    let serving = serving_runtime(root.path(), allocation_bytes);
    let allocations = serving.physical_allocations();
    let observation = match workload {
        CertificationResidencyWorkload::Maintenance => {
            let _allocation = allocations
                .admit_maintenance(allocation_bytes)
                .expect("real Store maintenance allocation should admit");
            serving.residency_observation()
        }
    };
    let closed = serving.close();
    assert!(!closed.residency().requires_inspection());
    observation
}

fn serving_runtime(root: &std::path::Path, allocation_bytes: NonZeroU64) -> ServingPhysicalRuntime {
    let runtime = PhysicalStore::admit(
        PhysicalRuntimeAdmission::new(root).expect("temporary Store root should declare"),
    )
    .expect("temporary Store runtime should admit");
    let media = match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("temporary Store media should admit"),
    };
    let (format, placement, access) = record_configuration();
    let durability = durability_policy(&media);
    let request = PhysicalRecordInitialization::new(format, placement, access, durability)
        .with_residency_policy(residency_policy(format, allocation_bytes));
    match media.initialize_record_store(request).into_raw() {
        TransitionOutcome::Success(serving) => serving,
        _ => panic!("temporary Store record runtime should initialize"),
    }
}

fn durability_policy(media: &MediaOwnedPhysicalRuntime) -> AdmittedPhysicalDurabilityPolicy {
    let basis = media
        .physical_durability_admission_basis()
        .expect("qualified certification media should expose a durability basis");
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(nonzero_frames(32)),
            GroupCommitDelay::new(nonzero_bytes(1)),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(nonzero_bytes(8 * 1024 * 1024)),
            WalSegmentInventoryLimit::new(nonzero_frames(1_024)),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(nonzero_bytes(4)),
            PendingUnresolvedMutationLimit::new(nonzero_frames(1_024)),
            LiveIdempotencyBindingLimit::new(nonzero_frames(4_096)),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(nonzero_bytes(16 * 1024 * 1024)),
            RetainedWalTailLimit::new(nonzero_bytes(64 * 1024 * 1024)),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy,
        _ => panic!("certification durability policy should admit"),
    }
}

fn record_configuration() -> (
    AdmittedPhysicalRecordFormat,
    AdmittedRecordPlacementPolicy,
    AdmittedRecordAccessPolicy,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    (format, placement, access)
}

fn residency_policy(
    format: AdmittedPhysicalRecordFormat,
    allocation_bytes: NonZeroU64,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    const METADATA_BYTES: u64 = 256 * 1024;
    const FRAME_BYTES: u64 = 16 * 1024;
    let scope_bytes = allocation_bytes.get().max(FRAME_BYTES);
    let total_bytes = scope_bytes + METADATA_BYTES + (FRAME_BYTES * 2);
    let mut declaration = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero_bytes(total_bytes))
        .resident_bytes(nonzero_bytes(FRAME_BYTES))
        .metadata_bytes(nonzero_bytes(METADATA_BYTES))
        .frame_entries(nonzero_frames(8))
        .pinned_frames(nonzero_frames(8))
        .pin_leases(nonzero_frames(8))
        .dirty_frames(nonzero_frames(4))
        .dirty_replacement_bytes(nonzero_bytes(FRAME_BYTES))
        .operation_bytes(nonzero_bytes(scope_bytes));
    for scope in [
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
    ] {
        declaration = declaration.scope_bytes(scope, nonzero_bytes(scope_bytes));
    }
    for kind in [
        PhysicalSpeculativeWorkKind::Prefetch,
        PhysicalSpeculativeWorkKind::ReadAhead,
        PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        declaration = declaration.speculative_frames(kind, nonzero_frames(1));
    }
    declaration
        .admit(format)
        .into_result()
        .expect("certification Store residency policy should admit")
}

fn nonzero_bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("certification byte declarations are non-zero")
}

fn nonzero_frames(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("certification frame declarations are non-zero")
}

#[cfg(test)]
mod tests {
    use super::{observe_real_store_residency, CertificationResidencyWorkload};
    use worth_store::physical_runtime::PhysicalOperationAllocationScope;

    #[test]
    fn workload_allocation_is_distinct_from_store_startup_peak() {
        let observation = observe_real_store_residency(
            "physical-certification-startup-peak",
            CertificationResidencyWorkload::Maintenance,
            1,
        );
        let counters = observation.counters();

        assert_eq!(counters.active_operation_bytes(), 1);
        assert_eq!(
            counters.peak_operation_bytes_for(PhysicalOperationAllocationScope::Maintenance),
            1
        );
        assert!(counters.peak_operation_bytes() > counters.active_operation_bytes());
    }
}
