use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint as Checkpoint;
use worth_store::physical_runtime::PhysicalIntegrityScrubDeferral;
use {
    std::time::Duration,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest as Request,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store_physical_integrity::PhysicalIntegrityObservationOutcome,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
    worth_store_testkit_store::integrity_scrub::assert_released,
    worth_store_testkit_store::integrity_scrub::request,
    worth_store_testkit_store::integrity_scrub::target,
};

#[test]
fn cancel_after_effect_keeps_live_resources_and_the_completed_window() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let mut first = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let mut second = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let cancellation = first.cancellation();
    let gate = serving
        .certification_pause_physical_execution_at(worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement);
    let worker = std::thread::spawn(move || {
        let observation = first.next_window();
        (first, observation)
    });
    assert!(gate.await_arrival());
    assert_eq!(
        serving.physical_scheduler_capacity().active_reservations(),
        1
    );
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub
            ),
        target.declared_bytes() as u64
    );
    assert!(matches!(
        second.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Deferred(
            worth_store::physical_runtime::PhysicalIntegrityScrubDeferral::AnotherWindowActive
        )
    ));
    cancellation.cancel();
    gate.release();
    let (mut first, observation) = worker.join().unwrap();
    assert!(
        matches!(observation, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation) if observation.counters.completed_windows == 1 && observation.counters.acquired_bytes == target.declared_bytes() as u64)
    );
    assert!(
        matches!(first.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(counters) if counters.completed_windows == 1)
    );
    assert_released(&serving);
    assert!(matches!(
        second.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(_)
    ));
    serving.close();
}

#[test]
fn close_waits_for_in_flight_window_before_releasing_media() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement);
    let reader = std::thread::spawn(move || {
        let observation = handle.next_window();
        (handle, observation)
    });
    assert!(gate.await_arrival());
    let (closed_tx, closed_rx) = std::sync::mpsc::channel();
    let closer = std::thread::spawn(move || {
        serving.close();
        closed_tx.send(()).unwrap();
    });
    assert!(
        closed_rx
            .recv_timeout(std::time::Duration::from_millis(50))
            .is_err(),
        "close cannot tear down paused IO"
    );
    gate.release();
    let (mut handle, observation) = reader.join().unwrap();
    closer.join().unwrap();
    closed_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    assert!(
        matches!(observation, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation) if observation.counters.acquired_bytes == target.declared_bytes() as u64)
    );
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Closed(counters) if counters.completed_windows == 1)
    );
}

#[test]
fn deadline_after_effect_counts_bytes_but_does_not_claim_intact() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        std::time::Duration::from_secs(1),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement);
    let worker = std::thread::spawn(move || {
        let observation = handle.next_window();
        (handle, observation)
    });
    assert!(gate.await_arrival());
    std::thread::sleep(std::time::Duration::from_millis(1050));
    gate.release();
    let (mut handle, observation) = worker.join().unwrap();
    assert!(
        matches!(observation, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Indeterminate(_)))
        && observation.counters.acquired_bytes == target.declared_bytes() as u64
        && observation.validation_counters.inspected_frames() == 0)
    );
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::DeadlineExceeded(counters) if counters.indeterminate_windows == 1)
    );
    assert_released(&serving);
    serving.close();
}
