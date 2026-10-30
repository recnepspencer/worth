#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::durability_admission::checkpoint_lifecycle::*;
use {std::path::Path};
use {std::time::Duration, std::time::Instant};

use worth_signal::facade::TemporalDuration;
use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint,
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionPauseGate,
    worth_store::physical_runtime::certification::MediaFaultDirective,
};
use {
    worth_store::physical_runtime::FilesystemMediaAdmission,
    worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointHandle,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointProgressPhase,
    worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalCheckpointStartDeferred,
    worth_store::physical_runtime::PhysicalCheckpointStartStale,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::PhysicalStoreCloseOutcome,
    worth_store::physical_runtime::PhysicalWalGroupAppendOutcome,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_backend::CertificationMediaFaultActivation,
    worth_store_physical_backend::FilesystemAccessPosture,
    worth_store_physical_backend::MediaOperationRole,
};

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::durability,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn active_checkpoint_joins_same_key_and_defers_distinct_key_without_parallel_effects() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 111);
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );

    let first = start(&serving, checkpoint_request(11));
    assert!(gate.await_arrival());
    let joined = start(&serving, checkpoint_request(11));
    assert_eq!(joined.identity(), first.identity());
    assert!(matches!(
        serving
            .checkpoints()
            .start(checkpoint_request(12))
            .into_raw(),
        worth_proof::TransitionOutcome::Deferred(
            worth_store::physical_runtime::PhysicalCheckpointStartDeferred::CaptureAlreadyActive
        )
    ));

    let identity = joined.identity();
    drop(first);
    gate.release();
    assert!(matches!(
        joined.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed)
            if completed.basis().identity() == identity
    ));
    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().started(), 1);
    assert_eq!(shutdown.checkpoint().completed(), 1);
    assert_eq!(shutdown.checkpoint().latest_publication(), Some(identity));
}

#[test]
fn accepted_cancellation_reconciles_the_exact_candidate_through_media_delete() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 112);
    let before = serving.media_counters();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(21));
    let identity = handle.identity();
    assert!(gate.await_arrival());

    assert_eq!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome::Accepted { identity }
    );
    let creation_release = gate.release_arrival(0).unwrap();
    assert!(creation_release.await_resumption());
    assert!(gate.await_arrivals(2));
    assert_eq!(
        handle.progress().phase(),
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::CandidateCleanup
    );
    gate.release();
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(no_effect)
            if no_effect.identity() == identity
                && no_effect.cause()
                    == worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
    ));

    let after = serving.media_counters();
    assert_eq!(
        after.identified_operation_attempts_for(
            worth_store_physical_backend::MediaOperationRole::Delete
        ) - before.identified_operation_attempts_for(
            worth_store_physical_backend::MediaOperationRole::Delete
        ),
        1
    );
    assert!(
        after.completed_operations_for(worth_store_physical_backend::MediaOperationRole::Delete)
            > before
                .completed_operations_for(worth_store_physical_backend::MediaOperationRole::Delete)
    );
    assert_eq!(
        after.partial_effects_for(worth_store_physical_backend::MediaOperationRole::Delete),
        before.partial_effects_for(worth_store_physical_backend::MediaOperationRole::Delete)
    );
    assert_eq!(
        after.indeterminate_effects_for(worth_store_physical_backend::MediaOperationRole::Delete),
        before.indeterminate_effects_for(worth_store_physical_backend::MediaOperationRole::Delete)
    );
    assert!(!candidate_path(&store_root, identity.sequence().get()).exists());
    assert!(!store_root.join("families/checkpoint.current").exists());

    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().proven_no_effect(), 1);
    assert_eq!(shutdown.checkpoint().latest_publication(), None);
    assert!(!shutdown.checkpoint().requires_inspection());
}

#[test]
fn later_no_effect_attempt_does_not_erase_retained_publication_authority() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 114);
    let first = start(&serving, checkpoint_request(41));
    let published = match first.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => {
            completed.basis().identity()
        }
        _ => panic!("the first checkpoint must publish"),
    };

    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let second = start(&serving, checkpoint_request(42));
    assert!(gate.await_arrival());
    assert!(matches!(
        second.request_cancellation(),
        worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome::Accepted { .. }
    ));
    gate.release();
    assert!(matches!(
        second.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(_)
    ));

    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().started(), 2);
    assert_eq!(shutdown.checkpoint().completed(), 1);
    assert_eq!(shutdown.checkpoint().proven_no_effect(), 1);
    assert_eq!(shutdown.checkpoint().latest_publication(), Some(published));
}

#[test]
fn cleanup_stabilizes_an_exact_candidate_that_is_already_absent() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 115);
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(51));
    let identity = handle.identity();
    assert!(gate.await_arrival());
    assert!(matches!(
        handle.request_cancellation(),
        worth_store::physical_runtime::PhysicalCheckpointCancellationOutcome::Accepted { .. }
    ));
    let creation_release = gate.release_arrival(0).unwrap();
    assert!(creation_release.await_resumption());
    assert!(gate.await_arrivals(2));

    let candidate = candidate_path(&store_root, identity.sequence().get());
    assert!(candidate.exists());
    std::fs::remove_file(&candidate).unwrap();
    gate.release();
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::ProvenNoEffect(no_effect)
            if no_effect.cause()
                == worth_store::physical_runtime::PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
    ));
    assert!(!candidate.exists());
    assert!(!serving.close().checkpoint().requires_inspection());
}

#[test]
fn indeterminate_candidate_creation_requires_inspection_when_cleanup_cannot_be_proved() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, activation) = fault_scheduled_serving(&store_root);
    append_durable_wal(&serving, 116);
    activation.arm().unwrap();

    let handle = start(&serving, checkpoint_request(61));
    let identity = handle.identity();
    let outcome = handle.wait();
    assert!(
        matches!(
            outcome,
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Indeterminate(ref indeterminate)
                if indeterminate.failure()
                    == worth_store::physical_runtime::PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed
        ),
        "candidate-create fault settled as {outcome:?}"
    );
    assert!(activation.is_consumed());
    assert!(candidate_path(&store_root, identity.sequence().get()).exists());
    assert!(!store_root.join("families/checkpoint.current").exists());
    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().indeterminate(), 1);
    assert!(shutdown.checkpoint().requires_inspection());
}

#[test]
fn close_drains_store_owned_checkpoint_after_the_caller_drops_its_handle() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 113);
    let retained_submission = serving.checkpoints();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(31));
    let identity = handle.identity();
    assert!(gate.await_arrival());
    drop(handle);

    let plan = serving.close_plan();
    let progress = plan.observation();
    let closing = std::thread::spawn(move || plan.execute());
    await_checkpoint_admission_stop(&retained_submission);
    assert_eq!(progress.completed_phase_count(), 0);

    gate.release();
    let closed = closing.join().unwrap();
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    assert_eq!(closed.shutdown().checkpoint().started(), 1);
    assert_eq!(closed.shutdown().checkpoint().proven_no_effect(), 1);
    assert_eq!(closed.shutdown().checkpoint().latest_publication(), None);
    assert!(!candidate_path(&store_root, identity.sequence().get()).exists());
}
#[path = "observation/mod.rs"]
pub(crate) mod observation;
#[path = "scrub_source/mod.rs"]
pub(crate) mod scrub_source;
