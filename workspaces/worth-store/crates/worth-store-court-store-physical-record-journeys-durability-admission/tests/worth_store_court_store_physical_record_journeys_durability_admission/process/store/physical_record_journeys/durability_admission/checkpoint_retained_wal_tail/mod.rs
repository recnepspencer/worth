#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_retained_wal_tail::*;
use {std::sync::mpsc, std::sync::mpsc::TryRecvError};
use {std::time::Duration, std::time::Instant};

use {worth_proof::TransitionOutcome};
use {worth_signal::facade::TemporalDuration};
use {worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission};
use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint, worth_store::physical_runtime::certification::CertificationPhysicalExecutionPauseGate};
use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::CompletedPhysicalCheckpoint, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointHandle, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointProgressPhase, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PhysicalWalReclamationObservation, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_format::PhysicalCheckpointSource};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::pause_checkpoint_at_phase};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::append_group, worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::prepared, worth_store_testkit_store::physical_record_journeys::durability_admission::wal_rotation::wal_policy};

#[test]
fn published_checkpoint_carries_the_exact_original_multi_rotation_wal_tail() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(1, b"checkpoint-base")]);
    let (completed, frozen_source) =
        publish_after_foreground_rotations(&serving, submission, placement);
    assert_independent_tail(&store_root, frozen_source, &completed);
    assert_reclaimed_prefix(&completed, 1);

    serving.close();
}

#[test]
fn foreground_wal_cannot_cross_the_checkpoint_publication_cutover() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(31, b"cutover-base")]);
    let (gate, handle, publication_arrival) = checkpoint_at_publication_replacement(&serving);
    let frozen_source = handle.source();
    let (foreground, completion) = start_cutover_foreground(submission, placement);
    assert!(matches!(completion.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));
    let completed = finish_checkpoint_cutover(&gate, handle, publication_arrival, &completion);
    assert!(matches!(completion.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)));

    let tail = completed.retained_wal_tail();
    let boundary = frozen_source.wal().covered_end_lsn_exclusive();
    assert_eq!(tail.checkpoint_boundary_lsn().get(), boundary);
    assert_eq!(tail.durable_tail_end_lsn_exclusive().get(), boundary);
    assert_eq!(tail.segments().len(), 1);
    assert!(matches!(
        completed.wal_reclamation(),
        worth_store::physical_runtime::PhysicalWalReclamationObservation::NotRequired { .. }
    ));

    gate.release();
    let foreground_end = completion.recv().unwrap();
    foreground.join().unwrap();
    assert!(foreground_end > tail.durable_tail_end_lsn_exclusive().get());
    serving.close();
}
