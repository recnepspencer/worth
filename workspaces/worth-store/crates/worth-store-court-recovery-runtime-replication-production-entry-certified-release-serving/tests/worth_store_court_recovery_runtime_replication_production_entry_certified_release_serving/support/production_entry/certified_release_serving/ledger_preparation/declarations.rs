//! A genuine recovered seal enters Serving only under the residency policy its
//! recovery was funded by; a different Serving policy is refused before any
//! publication, and the same media then reopens under the original policy.
//!
//! The Serving open policy must equal the recovery policy
//! (`PhysicalResidencyOwner::validate_recovered_policy` returns
//! `RecoveredResidencyPolicyMismatch`), so a world with an adequate recovery
//! policy and a tighter Serving ceiling cannot be built. The combined
//! seal-plus-bootstrap resident ceiling is unit-tested by
//! `recovery_residency::tests::retained_state_and_reducing_ceiling_precede_allocation`.

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
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation,
    worth_store::physical_runtime::PhysicalStore, worth_store::physical_runtime::PhysicalWalPolicy,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecoveredPhysicalCheckpointCustody,
    worth_store::physical_runtime::RetainedWalTailLimit,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};
pub(crate) use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

pub(crate) use {worth_proof::TransitionOutcome, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery::recover, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request, worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory};

pub(crate) fn recover_seal(
    root: &Path,
    original_recovery_bytes: u64,
) -> RecoveredPhysicalCheckpointCustody {
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request_with_memory(root, original_recovery_bytes));
    let handoff = match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
            "genuine C8 blocked: kind={:?}, planning={:?}, publication={:?}, effects={}",
            blocked.cause(),
            blocked.evidence().planning_denial,
            blocked.evidence().publication_denial,
            blocked.recovery_effects()
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Refused(refused) => panic!(
            "genuine C8 refused: kind={:?}, effects={}",
            refused.kind,
            refused.recovery_effects()
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
            "genuine C8 publication indeterminate: reopen={:?}, handoff={:?}, checkpoint_residue={}, counters={:?}, effects={}",
            failure.reopen_failure(),
            failure.handoff_failure(),
            failure.checkpoint_residue_indeterminate(),
            failure.counters(),
            failure.recovery_effects()
        ),
    };
    let core = handoff.into_core();
    assert_eq!(
        core.recovery_allocation_admission().byte_limit(),
        original_recovery_bytes
    );
    core.certification_release_media_into_checkpoint_custody()
        .expect("Store rejoined release grants a one-shot seal")
}

pub(crate) fn assert_policy_mismatch_refusal(
    root: &Path,
    seal: RecoveredPhysicalCheckpointCustody,
    target: u64,
) {
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
    let request =
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(tight_policy(format, target))
            .with_recovered_checkpoint_custody(seal);
    let worth_proof::TransitionOutcome::Denied(denial) =
        media.open_record_store(request).into_raw()
    else {
        panic!("a Serving policy other than the recovery policy must deny")
    };
    let reason = denial.reason();
    let release = denial.into_runtime().close();
    assert!(matches!(
        release,
        worth_store::physical_runtime::MediaShutdownOutcome::Released(_)
    ));
    assert!(
        matches!(
            reason,
            worth_store::physical_runtime::RecordBootstrapDenial::RecoveredResidencyPolicyMismatch
        ),
        "expected the recovered residency policy refusal, got {reason:?}"
    );
}

pub(crate) fn tight_policy(
    format: AdmittedPhysicalRecordFormat,
    target: u64,
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
        .operation_bytes(bytes(target))
        .progress_headroom_bytes(0);
    for scope in [
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
    ] {
        policy = policy.scope_bytes(scope, bytes(target));
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
        .expect("finite tight target policy must admit")
}
