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

use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, CheckpointMemoryLimit, FilesystemAccessPosture,
    FilesystemMediaAdmission, GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy,
    PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy,
    PhysicalOperationAllocationScope as Scope, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordOpen, PhysicalRecordResidencyPolicy,
    PhysicalRuntimeAdmission, PhysicalSpeculativeWorkKind as Speculation, PhysicalStore,
    PhysicalWalPolicy, RecordBootstrapDenial, RecoveredPhysicalCheckpointCustody,
    RetainedWalTailLimit, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

use super::*;

#[path = "ledger_preparation/world.rs"]
mod world;

#[test]
fn recovered_seal_refuses_a_different_serving_policy_then_reopens_under_its_own() {
    let worker = std::thread::Builder::new()
        .name("recovered-serving-ledger-preparation".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let world = world::checkpointed_partial_release_world();
            let producer_policy = world.serving().residency_observation().admitted_policy();
            // Size this fixture from the producer's admitted scope. Fresh C8
            // still obtains its own allocation authority for this request.
            let original_recovery_bytes = producer_policy
                .operation_bytes()
                .min(producer_policy.scope_bytes(Scope::Recovery));
            let retained = world.retained_root();
            let root = retained.path().to_path_buf();
            drop(world);

            let selected_checkpoint = root.join("families/checkpoint.current");
            let before = std::fs::read(&selected_checkpoint).expect("selected checkpoint bytes");
            let selected_root = root.join("families/records/root-current.selector");
            let root_before = std::fs::read(&selected_root).expect("selected root selector bytes");
            let seal = recover_seal(&root, original_recovery_bytes);
            assert_policy_mismatch_refusal(&root, seal, original_recovery_bytes / 2);
            assert_eq!(
                std::fs::read(&selected_checkpoint).expect("checkpoint after refusal"),
                before,
                "policy refusal must precede a new publication"
            );
            assert_eq!(
                std::fs::read(&selected_root).expect("root selector after refusal"),
                root_before,
                "policy refusal must preserve the selected root"
            );

            // The seal was consumed by the attempted open. A fresh C8/Store
            // rejoin under the original policy still opens and checkpoints
            // through the ordinary production path.
            super::open_serving_with_seal(&root, recover_seal(&root, original_recovery_bytes));
        })
        .expect("recovery worker");
    worker.join().expect("recovery worker did not panic");
}

fn recover_seal(root: &Path, original_recovery_bytes: u64) -> RecoveredPhysicalCheckpointCustody {
    let outcome =
        WorthStoreRecovery::recover(super::request_with_memory(root, original_recovery_bytes));
    let handoff = match outcome {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff,
        PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
            "genuine C8 blocked: kind={:?}, planning={:?}, publication={:?}, effects={}",
            blocked.cause(),
            blocked.evidence().planning_denial,
            blocked.evidence().publication_denial,
            blocked.recovery_effects()
        ),
        PhysicalRecoveryOutcome::Refused(refused) => panic!(
            "genuine C8 refused: kind={:?}, effects={}",
            refused.kind,
            refused.recovery_effects()
        ),
        PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
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
    core.into_checkpoint_custody()
        .expect("Store rejoined release grants a one-shot seal")
}

fn assert_policy_mismatch_refusal(
    root: &Path,
    seal: RecoveredPhysicalCheckpointCustody,
    target: u64,
) {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("Serving media must admit")
    };
    let TransitionOutcome::Success(durability) = PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 << 20).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 << 20).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    else {
        panic!("Serving durability must admit")
    };
    let request = PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(tight_policy(format, target))
        .with_recovered_checkpoint_custody(seal);
    let TransitionOutcome::Denied(denial) = media.open_record_store(request).into_raw() else {
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
            RecordBootstrapDenial::RecoveredResidencyPolicyMismatch
        ),
        "expected the recovered residency policy refusal, got {reason:?}"
    );
}

fn tight_policy(
    format: AdmittedPhysicalRecordFormat,
    target: u64,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    let bytes = |value| NonZeroU64::new(value).unwrap();
    let count = |value| NonZeroU32::new(value).unwrap();
    let mut policy = PhysicalRecordResidencyPolicy::builder()
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
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        policy = policy.scope_bytes(scope, bytes(target));
    }
    policy
        .speculative_frames(Speculation::Prefetch, count(256))
        .speculative_frames(Speculation::ReadAhead, count(256))
        .speculative_frames(Speculation::WriteBehind, count(64))
        .admit(format)
        .into_result()
        .expect("finite tight target policy must admit")
}
