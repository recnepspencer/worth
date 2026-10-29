#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::checkpoint_pin_budget::*;
use {std::fs, std::num::NonZeroU32, std::num::NonZeroU64, std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::CheckpointMemoryLimit, worth_store::physical_runtime::GroupCommitDelay, worth_store::physical_runtime::GroupCommitLimit, worth_store::physical_runtime::IdempotencyRetentionGenerations, worth_store::physical_runtime::LiveIdempotencyBindingLimit, worth_store::physical_runtime::PendingUnresolvedMutationLimit, worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointPolicy, worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalDurabilityDeclaration, worth_store::physical_runtime::PhysicalIdempotencyPolicy, worth_store::physical_runtime::PhysicalRecordAccessPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalWalPolicy, worth_store::physical_runtime::RetainedWalTailLimit, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::WalSegmentByteLimit, worth_store::physical_runtime::WalSegmentInventoryLimit};

use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement};

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
    let published = std::fs::read(&checkpoint_path).unwrap();
    let mut session = serving
        .records()
        .unwrap()
        .open(first, worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits())
        .unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        b"ordinary-checkpoint-record"
    );
    drop(session);

    append(&serving, policy, 2, b"ordinary-record-after-checkpoint");
    checkpoint(&serving, 2);
    assert_ne!(std::fs::read(&checkpoint_path).unwrap(), published);
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
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("low-budget checkpoint must start"),
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(effect)
            if effect.cause()
                == worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                    worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable
                )
    ));
    assert!(!checkpoint_path.exists());
    serving.close();
}
