use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
use {worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome, worth_store::physical_runtime::PhysicalCheckpointDisposal, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointPoll, worth_store::physical_runtime::PhysicalCheckpointProgressPhase, worth_store::physical_runtime::PhysicalStoreCloseOutcome};

use {worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::await_checkpoint_admission_stop, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::checkpoint_request, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::pause_checkpoint_at_phase, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::serving_with_durable_wal, worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::start};

#[test]
fn pending_disposal_abandons_only_observation_and_close_drains_the_attempt() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 117);
    let retained_submission = serving.checkpoints();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(71));
    let identity = handle.identity();
    assert!(gate.await_arrival());
    assert_eq!(
        handle.dispose(),
        worth_store::physical_runtime::PhysicalCheckpointDisposal::ObservationAbandoned { identity }
    );

    let plan = serving.close_plan();
    let closing = std::thread::spawn(move || plan.execute());
    await_checkpoint_admission_stop(&retained_submission);
    gate.release();
    let closed = closing.join().unwrap();

    assert!(matches!(closed, worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().checkpoint().started(), 1);
    assert_eq!(closed.shutdown().checkpoint().proven_no_effect(), 1);
    assert!(!closed.shutdown().checkpoint().requires_inspection());
}

#[test]
fn terminal_poll_and_disposal_return_the_exact_store_finalized_outcome() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_with_durable_wal(&parent.path().join("store"), 118);
    let waiting = start(&serving, checkpoint_request(81));
    let observing = start(&serving, checkpoint_request(81));
    assert_eq!(waiting.identity(), observing.identity());

    let terminal = waiting.wait();
    assert!(matches!(terminal, worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)));
    assert_eq!(
        observing.poll(),
        worth_store::physical_runtime::PhysicalCheckpointPoll::Terminal(terminal.clone())
    );
    assert_eq!(
        observing.dispose(),
        worth_store::physical_runtime::PhysicalCheckpointDisposal::Terminal(terminal)
    );

    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().started(), 1);
    assert_eq!(shutdown.checkpoint().completed(), 1);
    assert_eq!(shutdown.checkpoint().worker_panics(), 0);
}

#[test]
fn cancellation_after_publication_cutover_cannot_claim_no_effect() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 119);
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(91));
    let identity = handle.identity();
    pause_checkpoint_at_phase(
        &gate,
        &handle,
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::PublicationReplacement,
    );
    assert_eq!(
        handle.progress().phase(),
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::PublicationReplacement
    );
    assert_eq!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome::PublicationAlreadyEffectful { identity }
    );

    gate.release();
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    assert!(store_root.join("families/checkpoint.current").exists());
    assert_eq!(serving.close().checkpoint().completed(), 1);
}
