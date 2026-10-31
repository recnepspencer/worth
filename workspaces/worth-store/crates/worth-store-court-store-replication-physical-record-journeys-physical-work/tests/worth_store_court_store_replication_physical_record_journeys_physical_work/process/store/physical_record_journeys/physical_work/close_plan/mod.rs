use tempfile::tempdir;

mod task;

use {
    crate::physical_record_journeys::physical_work::readiness::success,
    worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::foreground_saturation_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
    worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work,
};

#[test]
fn close_safely_cancels_predispatch_work_and_reconciles_signal() {
    let root = tempdir().unwrap();
    let (profile, requests) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let retained_submission = serving.physical_mutation_submission();
    let [shared, queued_request, settled_request, cancelled_request] = requests;

    let declared = success(retained_submission.submit(shared.clone()));
    let ready = ready_work(&serving, shared.clone());
    let ready_identity = ready.intent().identity();
    let queued = admitted_write(&serving, queued_request);
    let queued_identity = queued.intent().identity();
    let settled = admitted_write(&serving, settled_request);
    let settled_identity = settled.intent().identity();
    let settled = serving
        .execute_physical_work(
            worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
                settled,
                b"settled!".as_slice(),
            )
            .unwrap(),
        )
        .unwrap();
    let cancelled = admitted_write(&serving, cancelled_request);
    let cancelled_consumer = cancelled.consumer_handle();
    let cancelled_command = worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(
        cancelled,
        b"cancelld".as_slice(),
    )
    .unwrap();
    serving.cancel_physical_work(cancelled_consumer).unwrap();

    let close_plan = serving.close_plan();
    let progress = close_plan.observation();
    assert_eq!(progress.completed_phase_count(), 0);
    assert_eq!(progress.latest_phase(), None);
    let closed = close_plan.execute();

    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    assert_eq!(
        closed.phases(),
        &[
            worth_store::physical_runtime::PhysicalStoreClosePhase::CheckpointDrained,
            worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped,
            worth_store::physical_runtime::PhysicalStoreClosePhase::SafeCancellationComplete,
            worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete,
            worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed,
            worth_store::physical_runtime::PhysicalStoreClosePhase::ResidencyClosed,
            worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased,
        ]
    );
    assert_eq!(progress.completed_phase_count(), 7);
    assert_eq!(
        progress.latest_phase(),
        Some(worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased)
    );
    assert!(closed.phases().iter().all(|phase| progress.reached(*phase)));
    let drain = closed.shutdown().work().drain();
    assert_eq!(drain.settled(), &[settled_identity]);
    assert_eq!(
        drain.cancelled_before_dispatch(),
        &[
            declared.identity(),
            ready_identity,
            queued_identity,
            cancelled_consumer.identity(),
        ]
    );
    assert!(drain.residual().is_empty());
    assert_eq!(
        closed
            .shutdown()
            .signal_summary()
            .unwrap()
            .active_in_flight_node_count(),
        0
    );
    assert!(matches!(
        retained_submission.submit(shared).into_raw(),
        worth_proof::TransitionOutcome::Stale(
            worth_store::physical_runtime::PhysicalWorkSubmissionStale::OwnerReleased
        )
    ));
    drop((ready, queued, settled, cancelled_command));
}

#[test]
fn abort_returns_a_distinct_terminal_outcome() {
    let root = tempdir().unwrap();
    let (profile, _, request) = work_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let receipt = success(serving.physical_mutation_submission().submit(request));

    let aborted = serving.abort_with_evidence();

    assert_eq!(
        aborted.phases().last(),
        Some(&worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased)
    );
    assert_eq!(
        aborted
            .shutdown()
            .work()
            .drain()
            .cancelled_before_dispatch(),
        &[receipt.identity()]
    );
    assert!(!aborted.requires_inspection());
}

#[test]
fn certification_close_gate_holds_lifecycle_after_signal_disposal() {
    let root = tempdir().unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), work_fixture().0);
    let plan = serving.close_plan();
    let progress = plan.observation();
    let gate = plan.certification_pause_at(
        worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed,
    );
    let closing = std::thread::spawn(move || plan.execute());

    assert!(gate.await_arrival());
    assert!(
        progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed)
    );
    assert!(
        !progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::ResidencyClosed)
    );
    assert!(
        !progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased)
    );

    gate.release();
    let closed = closing.join().unwrap();
    assert!(matches!(
        closed,
        worth_store::physical_runtime::PhysicalStoreCloseOutcome::Closed { .. }
    ));
    assert!(progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased));
}

#[test]
fn every_close_phase_survives_process_death_without_signal_replay_input() {
    let phases = [
        worth_store::physical_runtime::PhysicalStoreClosePhase::CheckpointDrained,
        worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped,
        worth_store::physical_runtime::PhysicalStoreClosePhase::SafeCancellationComplete,
        worth_store::physical_runtime::PhysicalStoreClosePhase::DispatchSettlementComplete,
        worth_store::physical_runtime::PhysicalStoreClosePhase::SignalDisposed,
        worth_store::physical_runtime::PhysicalStoreClosePhase::ResidencyClosed,
        worth_store::physical_runtime::PhysicalStoreClosePhase::MediaReleased,
    ];
    for phase in phases {
        let parent = tempdir().unwrap();
        let root = parent.path().join("store");
        worth_store_testkit_store::physical_record_journeys::close_phase_crash::kill_writer_at(
            &root, phase,
        );
        let reopened = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child("close_phase_reopener", &root, None);
        assert!(
            reopened
                .lines()
                .any(|line| line.starts_with("C5_CLOSE_REOPENED ")),
            "fresh executable failed to reopen after {phase:?}"
        );
    }
}
