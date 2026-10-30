//! The real recovery composition transfers one pool owner, never a new pool.

pub(crate) use {std::num::NonZeroU64, std::path::Path, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_proof::TransitionOutcome, worth_store_recovery_runtime::WorthStoreRecovery, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, crate::production_entry::certified_release_serving::release_capacity_budget::snapshot_family};
pub(crate) use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalResidencyDimension as Dimension, worth_store::physical_runtime::RecoveredPhysicalRuntimeCore};

pub(crate) fn recover_core(root: &Path) -> RecoveredPhysicalRuntimeCore {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(root));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine release media must recover: {outcome:?}")
    };
    assert_eq!(handoff.core().recovery_effect_count(), 0);
    handoff.into_core()
}

pub(crate) fn checkpoint(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: [u8; 32],
) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("genuine release checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "release checkpoint: {outcome:?}"
    );
}

pub(crate) fn policy_with_different_recovery_scope(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use {
        worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Kind,
    };
    let canonical =
        worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy::canonical(format);
    let bytes = |value| std::num::NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(canonical.total_bytes()))
        .resident_bytes(bytes(canonical.resident_bytes()))
        .metadata_bytes(bytes(canonical.metadata_bytes()))
        .frame_entries(count(canonical.frame_entries()))
        .pinned_frames(count(canonical.pinned_frames()))
        .pin_leases(count(canonical.pin_leases()))
        .dirty_frames(count(canonical.dirty_frames()))
        .dirty_replacement_bytes(bytes(canonical.dirty_replacement_bytes()))
        .operation_bytes(bytes(canonical.operation_bytes()))
        .progress_headroom_bytes(64 << 10);
    for scope in [
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
    ] {
        let value = canonical.scope_bytes(scope)
            - u64::from(
                scope == worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            );
        builder = builder.scope_bytes(scope, bytes(value));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, count(canonical.speculative_frames(kind)));
    }
    builder
        .admit(format)
        .into_result()
        .expect("only Recovery scope differs")
}

pub(crate) fn open_with_policy(
    root: &Path,
    seal: worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    format: AdmittedPhysicalRecordFormat,
    policy: AdmittedPhysicalRecordResidencyPolicy,
) -> worth_store::physical_runtime::RecordStoreOpenOutcome {
    use std::num::NonZeroU32;
    use worth_store::physical_runtime::*;
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let worth_proof::TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("real Serving media must admit")
    };
    let worth_proof::TransitionOutcome::Success(durability) =
        PhysicalDurabilityDeclaration::builder()
            .group_commit(
                GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
                GroupCommitDelay::new(std::num::NonZeroU64::MIN),
            )
            .wal(PhysicalWalPolicy::segmented(
                WalSegmentByteLimit::new(std::num::NonZeroU64::new(16 << 20).unwrap()),
                WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
            ))
            .idempotency(PhysicalIdempotencyPolicy::new(
                IdempotencyRetentionGenerations::new(std::num::NonZeroU64::new(4).unwrap()),
                PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
                LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
            ))
            .checkpoint(PhysicalCheckpointPolicy::fuzzy(
                CheckpointMemoryLimit::new(std::num::NonZeroU64::new(16 << 20).unwrap()),
                RetainedWalTailLimit::new(std::num::NonZeroU64::new(64 << 20).unwrap()),
            ))
            .admit(media.physical_durability_admission_basis().unwrap())
            .into_raw()
    else {
        panic!("real Serving durability must admit")
    };
    media.open_record_store(
        PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(policy)
            .with_recovered_checkpoint_custody(seal),
    )
}
