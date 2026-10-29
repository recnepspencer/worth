//! Selector and addressed-root funding failures use genuine released, checkpointed Store media.

pub(crate) use {std::num::NonZeroU64, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store::physical_runtime::PhysicalRecoveryRejoinResidentDenial, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, std::fs, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension as Dimension, worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Kind};
pub(crate) use {worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::ROOT_SELECTOR_BYTES};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryRootProtocolArtifact as Artifact, worth_store_recovery_runtime::PhysicalRecoverySourceDenial, worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationBoundary as Boundary, worth_store_recovery_runtime::PhysicalRecoverySourceReadAllocationDenial as Cause};

pub(crate) fn tiny_recovery_policy() -> AdmittedPhysicalRecordResidencyPolicy {
    recovery_policy_with_scope(1)
}

pub(crate) fn recovery_policy_with_scope(
    usable_recovery: u64,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let canonical = worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| std::num::NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    const PROGRESS_HEADROOM: u64 = 64 << 10;
    let mut builder = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
        .progress_headroom_bytes(PROGRESS_HEADROOM);
    for scope in [
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
    ] {
        let maximum = if scope == worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery {
            PROGRESS_HEADROOM + usable_recovery
        } else {
            canonical.scope_bytes(scope)
        };
        builder = builder.scope_bytes(scope, bytes(maximum));
    }
    for kind in [worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead, worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch, worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("actual policy with bounded usable Recovery bytes")
}