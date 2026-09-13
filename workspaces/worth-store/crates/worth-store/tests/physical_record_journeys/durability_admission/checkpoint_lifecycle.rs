use std::path::Path;
use std::time::{Duration, Instant};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::certification::{
    CertificationPhysicalExecutionCheckpoint, CertificationPhysicalExecutionPauseGate,
    MediaFaultDirective,
};
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalCheckpointCancellationOutcome, PhysicalCheckpointDeadline,
    PhysicalCheckpointHandle, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointProgressPhase, PhysicalCheckpointProvenNoEffectCause,
    PhysicalCheckpointRequest, PhysicalCheckpointStartDeferred, PhysicalCheckpointStartStale,
    PhysicalMutationIdempotencyMaterial, PhysicalRecordInitialization, PhysicalRuntimeAdmission,
    PhysicalStore, PhysicalStoreCloseOutcome, PhysicalWalGroupAppendOutcome,
    PhysicalWalGroupBarrierOutcome, ServingPhysicalRuntime,
};
use worth_store_physical_backend::{
    CertificationMediaFaultActivation, FilesystemAccessPosture, MediaOperationRole,
};

use super::super::{configuration, durability, serving_from_initialization, success};

#[path = "checkpoint_lifecycle/observation.rs"]
mod observation;
#[path = "checkpoint_lifecycle/scrub_source.rs"]
mod scrub_source;

pub(super) fn pause_checkpoint_at_phase(
    gate: &CertificationPhysicalExecutionPauseGate,
    handle: &PhysicalCheckpointHandle,
    target: PhysicalCheckpointProgressPhase,
) -> usize {
    let mut arrival = 0;
    loop {
        assert!(gate.await_arrivals(arrival + 1));
        if handle.progress().phase() == target {
            return arrival;
        }
        assert!(arrival < 64, "checkpoint did not reach {target:?}");
        let release = gate.release_arrival(arrival).unwrap();
        assert!(release.await_resumption());
        arrival += 1;
    }
}

#[test]
fn active_checkpoint_joins_same_key_and_defers_distinct_key_without_parallel_effects() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 111);
    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
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
        TransitionOutcome::Deferred(PhysicalCheckpointStartDeferred::CaptureAlreadyActive)
    ));

    let identity = joined.identity();
    drop(first);
    gate.release();
    assert!(matches!(
        joined.wait(),
        PhysicalCheckpointOutcome::Completed(completed)
            if completed.basis().identity() == identity
    ));
    let shutdown = serving.close();
    assert_eq!(shutdown.checkpoint().started(), 1);
    assert_eq!(shutdown.checkpoint().completed(), 1);
    assert_eq!(shutdown.checkpoint().latest_publication(), Some(identity));
}

#[test]
fn accepted_cancellation_reconciles_the_exact_candidate_through_c4_delete() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_with_durable_wal(&store_root, 112);
    let before = serving.media_counters();
    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(21));
    let identity = handle.identity();
    assert!(gate.await_arrival());

    assert_eq!(
        handle.request_cancellation(),
        PhysicalCheckpointCancellationOutcome::Accepted { identity }
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
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect)
            if no_effect.identity() == identity
                && no_effect.cause()
                    == PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
    ));

    let after = serving.media_counters();
    assert_eq!(
        after.identified_operation_attempts_for(MediaOperationRole::Delete)
            - before.identified_operation_attempts_for(MediaOperationRole::Delete),
        1
    );
    assert!(
        after.completed_operations_for(MediaOperationRole::Delete)
            > before.completed_operations_for(MediaOperationRole::Delete)
    );
    assert_eq!(
        after.partial_effects_for(MediaOperationRole::Delete),
        before.partial_effects_for(MediaOperationRole::Delete)
    );
    assert_eq!(
        after.indeterminate_effects_for(MediaOperationRole::Delete),
        before.indeterminate_effects_for(MediaOperationRole::Delete)
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
        PhysicalCheckpointOutcome::Completed(completed) => completed.basis().identity(),
        _ => panic!("the first checkpoint must publish"),
    };

    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let second = start(&serving, checkpoint_request(42));
    assert!(gate.await_arrival());
    assert!(matches!(
        second.request_cancellation(),
        PhysicalCheckpointCancellationOutcome::Accepted { .. }
    ));
    gate.release();
    assert!(matches!(
        second.wait(),
        PhysicalCheckpointOutcome::ProvenNoEffect(_)
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
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start(&serving, checkpoint_request(51));
    let identity = handle.identity();
    assert!(gate.await_arrival());
    assert!(matches!(
        handle.request_cancellation(),
        PhysicalCheckpointCancellationOutcome::Accepted { .. }
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
        PhysicalCheckpointOutcome::ProvenNoEffect(no_effect)
            if no_effect.cause()
                == PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
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
            PhysicalCheckpointOutcome::Indeterminate(indeterminate)
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
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
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
    assert!(matches!(closed, PhysicalStoreCloseOutcome::Closed { .. }));
    assert_eq!(closed.shutdown().checkpoint().started(), 1);
    assert_eq!(closed.shutdown().checkpoint().proven_no_effect(), 1);
    assert_eq!(closed.shutdown().checkpoint().latest_publication(), None);
    assert!(!candidate_path(&store_root, identity.sequence().get()).exists());
}

fn serving_with_durable_wal(root: &Path, material: u8) -> ServingPhysicalRuntime {
    let serving = serving_from_initialization(root);
    append_durable_wal(&serving, material);
    serving
}

fn append_durable_wal(serving: &ServingPhysicalRuntime, material: u8) {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let prepared = super::wal_append::prepared(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([material; 32]),
        b"checkpoint-lifecycle-redo",
    );
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("checkpoint lifecycle setup requires an appended WAL member"),
    };
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
}

fn fault_scheduled_serving(
    root: &Path,
) -> (ServingPhysicalRuntime, CertificationMediaFaultActivation) {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::PositionedWrite,
                1,
                MediaFaultDirective::IndeterminateAfterEffect,
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled checkpoint media admission must succeed"),
    };
    let (format, placement, access) = configuration();
    let policy = durability(&media);
    let serving = success(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, policy,
        )),
    );
    (serving, activation)
}

fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(
            TemporalDuration::temporal_duration(1_000).expect("deadline is positive"),
        ),
    )
}

fn start(
    serving: &ServingPhysicalRuntime,
    request: PhysicalCheckpointRequest,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    match serving.checkpoints().start(request).into_raw() {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint lifecycle setup requires an admitted handle"),
    }
}

fn await_checkpoint_admission_stop(
    submission: &worth_store::physical_runtime::PhysicalCheckpointSubmission,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if matches!(
            submission.start(checkpoint_request(32)).into_raw(),
            TransitionOutcome::Stale(PhysicalCheckpointStartStale::RuntimeClosing)
        ) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "checkpoint close did not stop admission"
        );
        std::thread::yield_now();
    }
}

fn candidate_path(root: &Path, sequence: u64) -> std::path::PathBuf {
    root.join(format!("staging/checkpoint-{sequence:016x}.candidate"))
}
