use std::{
    fs,
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, CheckpointMemoryLimit, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit,
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointDeadline,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointPolicy,
    PhysicalCheckpointProvenNoEffectCause, PhysicalCheckpointRequest,
    PhysicalDurabilityDeclaration, PhysicalIdempotencyPolicy, PhysicalRecordAccessPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordInitialization, PhysicalWalPolicy,
    RetainedWalTailLimit, ServingPhysicalRuntime, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

use super::{append, checkpoint, initialize, placement};

const MAINTENANCE_LIMIT: u64 = 16 * 1024 * 1024;

#[test]
fn ordinary_record_checkpoint_preserves_nested_scan_headroom() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize(&root);
    let policy = placement();
    let first = append(&serving, policy, 1, b"ordinary-checkpoint-record");

    assert_eq!(
        serving
            .durability_observation()
            .checkpoint_policy()
            .memory_limit()
            .get()
            .get(),
        MAINTENANCE_LIMIT,
    );
    checkpoint(&serving, 1);
    let checkpoint_path = root.join("families/checkpoint.current");
    let published = fs::read(&checkpoint_path).unwrap();
    let mut session = serving
        .records()
        .unwrap()
        .open(first, super::limits())
        .unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        b"ordinary-checkpoint-record"
    );
    drop(session);

    append(&serving, policy, 2, b"ordinary-record-after-checkpoint");
    checkpoint(&serving, 2);
    assert_ne!(fs::read(&checkpoint_path).unwrap(), published);
    serving.close();
}

#[test]
fn checkpoint_below_nested_scan_cost_denies_after_capture_without_publication() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize_with_checkpoint_budget(&root, 32 * 1024);
    append(&serving, placement(), 1, b"ordinary-low-budget-record");
    let checkpoint_path = root.join("families/checkpoint.current");
    assert!(!checkpoint_path.exists());
    let handle = match serving
        .checkpoints()
        .start(checkpoint_request(1))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("low-budget checkpoint must start"),
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::ProvenNoEffect(effect)
            if effect.cause()
                == PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                    PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable
                )
    ));
    assert!(!checkpoint_path.exists());
    serving.close();
}

fn initialize_with_checkpoint_budget(root: &Path, checkpoint_bytes: u64) -> ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::media(root);
    let durability = match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(NonZeroU32::new(64).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(checkpoint_bytes).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(media.physical_durability_admission_basis().unwrap())
        .into_raw()
    {
        TransitionOutcome::Success(policy) => policy,
        _ => panic!("low-budget durability must admit"),
    };
    super::super::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(
                format,
                super::placement_for(format),
                access,
                durability,
            )
            .with_residency_policy(super::residency(format, super::RESIDENT_BYTES)),
        ),
    )
}

fn checkpoint_request(ordinal: u64) -> PhysicalCheckpointRequest {
    let mut key = [0x6C; 32];
    key[..8].copy_from_slice(&ordinal.to_le_bytes());
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    )
}
