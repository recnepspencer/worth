//! Recovery carries one pool into Serving while retaining its original ceiling.
//! The larger or smaller target policy is declared before recovery admission.

pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::CheckpointMemoryLimit,
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::GroupCommitDelay,
    worth_store::physical_runtime::GroupCommitLimit,
    worth_store::physical_runtime::IdempotencyRetentionGenerations,
    worth_store::physical_runtime::LiveIdempotencyBindingLimit,
    worth_store::physical_runtime::PendingUnresolvedMutationLimit,
    worth_store::physical_runtime::PhysicalCheckpointPolicy,
    worth_store::physical_runtime::PhysicalDurabilityDeclaration,
    worth_store::physical_runtime::PhysicalIdempotencyPolicy,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    worth_store::physical_runtime::RetainedWalTailLimit,
    worth_store::physical_runtime::ServingPhysicalRuntime,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration, worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request, worth_store_testkit_recovery_runtime::production_entry::release_reopen, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration};

pub(crate) const ORIGINAL_RECOVERY_BYTES: u64 = 24 << 20;
pub(crate) const LOWER_TARGET_BYTES: u64 = 16 << 20;

pub(crate) fn recover_seal(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> RecoveredPhysicalCheckpointCustody {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_configuration(
        root,
        ORIGINAL_RECOVERY_BYTES,
        configuration,
    ));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("genuine selected release recovery denied: {outcome:?}")
    };
    let core = handoff.into_core();
    let admission = core.recovery_allocation_admission();
    assert_eq!(admission.store_identity(), store);
    assert_eq!(admission.byte_limit(), ORIGINAL_RECOVERY_BYTES);
    let seal = core
        .certification_release_media_into_checkpoint_custody()
        .expect("selected release grants a one-shot custody seal");
    assert_eq!(seal.recovery_allocation_admission(), admission);
    seal
}

pub(crate) fn checkpoint(serving: &ServingPhysicalRuntime, key: [u8; 32]) {
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("genuine released checkpoint must admit")
    };
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ),
        "genuine released checkpoint: {outcome:?}"
    );
}

pub(crate) fn open_with_lower_target_policy(
    root: &Path,
    seal: RecoveredPhysicalCheckpointCustody,
) -> ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root).unwrap(),
    )
    .unwrap();
    let worth_proof::TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(
            worth_store::physical_runtime::FilesystemMediaAdmission::production(
                worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
            ),
        )
        .into_raw()
    else {
        panic!("Serving media must admit")
    };
    let worth_proof::TransitionOutcome::Success(durability) =
        worth_store::physical_runtime::PhysicalDurabilityDeclaration::builder()
            .group_commit(
                worth_store::physical_runtime::GroupCommitLimit::new(
                    std::num::NonZeroU32::new(32).unwrap(),
                ),
                worth_store::physical_runtime::GroupCommitDelay::new(
                    std::num::NonZeroU64::new(1).unwrap(),
                ),
            )
            .wal(worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(16 << 20).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(1024).unwrap(),
                ),
            ))
            .idempotency(
                worth_store::physical_runtime::PhysicalIdempotencyPolicy::new(
                    worth_store::physical_runtime::IdempotencyRetentionGenerations::new(
                        std::num::NonZeroU64::new(4).unwrap(),
                    ),
                    worth_store::physical_runtime::PendingUnresolvedMutationLimit::new(
                        std::num::NonZeroU32::new(1024).unwrap(),
                    ),
                    worth_store::physical_runtime::LiveIdempotencyBindingLimit::new(
                        std::num::NonZeroU32::new(4096).unwrap(),
                    ),
                ),
            )
            .checkpoint(
                worth_store::physical_runtime::PhysicalCheckpointPolicy::fuzzy(
                    worth_store::physical_runtime::CheckpointMemoryLimit::new(
                        std::num::NonZeroU64::new(16 << 20).unwrap(),
                    ),
                    worth_store::physical_runtime::RetainedWalTailLimit::new(
                        std::num::NonZeroU64::new(64 << 20).unwrap(),
                    ),
                ),
            )
            .admit(media.physical_durability_admission_basis().unwrap())
            .into_raw()
    else {
        panic!("Serving durability must admit")
    };
    let open = worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(lower_target_policy(format))
        .with_recovered_checkpoint_custody(seal);
    let worth_proof::TransitionOutcome::Success(serving) = media.open_record_store(open).into_raw()
    else {
        panic!("smaller target policy must open from genuine custody seal")
    };
    serving
}

pub(crate) fn lower_target_policy(
    format: AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |value| std::num::NonZeroU64::new(value).unwrap();
    let count = |value| std::num::NonZeroU32::new(value).unwrap();
    let mut policy = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(384 << 20))
        .resident_bytes(bytes(64 << 20))
        .metadata_bytes(bytes(3 << 20))
        .frame_entries(count(4096))
        .pinned_frames(count(256))
        .pin_leases(count(512))
        .dirty_frames(count(64))
        .dirty_replacement_bytes(bytes(64 << 20))
        .operation_bytes(bytes(LOWER_TARGET_BYTES))
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
        policy = policy.scope_bytes(scope, bytes(LOWER_TARGET_BYTES));
    }
    policy
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch,
            count(256),
        )
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead,
            count(256),
        )
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind,
            count(64),
        )
        .admit(format)
        .into_result()
        .expect("finite lower target policy must admit")
}
