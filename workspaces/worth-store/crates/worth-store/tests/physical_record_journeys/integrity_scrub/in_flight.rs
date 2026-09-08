use super::*;
use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint as Checkpoint;
use worth_store::physical_runtime::PhysicalIntegrityScrubDeferral;

#[test]
fn cancel_after_effect_keeps_live_resources_and_the_completed_window() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let mut first = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let mut second = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let cancellation = first.cancellation();
    let gate = serving
        .certification_pause_physical_execution_at(Checkpoint::AfterReadBeforeSchedulerSettlement);
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
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Scrub),
        target.range().length() as u64
    );
    assert!(matches!(
        second.next_window(),
        Progress::Deferred(PhysicalIntegrityScrubDeferral::AnotherWindowActive)
    ));
    cancellation.cancel();
    gate.release();
    let (mut first, observation) = worker.join().unwrap();
    assert!(
        matches!(observation, Progress::WindowInspected(observation) if observation.counters.completed_windows == 1 && observation.counters.acquired_bytes == target.range().length() as u64)
    );
    assert!(
        matches!(first.next_window(), Progress::Cancelled(counters) if counters.completed_windows == 1)
    );
    assert_released(&serving);
    assert!(matches!(second.next_window(), Progress::WindowInspected(_)));
    serving.close();
}

#[test]
fn close_waits_for_in_flight_window_before_releasing_media() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(Checkpoint::AfterReadBeforeSchedulerSettlement);
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
        closed_rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "close cannot tear down paused IO"
    );
    gate.release();
    let (mut handle, observation) = reader.join().unwrap();
    closer.join().unwrap();
    closed_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        matches!(observation, Progress::WindowInspected(observation) if observation.counters.acquired_bytes == target.range().length() as u64)
    );
    assert!(
        matches!(handle.next_window(), Progress::Closed(counters) if counters.completed_windows == 1)
    );
}

#[test]
fn deadline_after_effect_counts_bytes_but_does_not_claim_intact() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let request = Request::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        Duration::from_secs(1),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(Checkpoint::AfterReadBeforeSchedulerSettlement);
    let worker = std::thread::spawn(move || {
        let observation = handle.next_window();
        (handle, observation)
    });
    assert!(gate.await_arrival());
    std::thread::sleep(Duration::from_millis(1050));
    gate.release();
    let (mut handle, observation) = worker.join().unwrap();
    assert!(matches!(observation, Progress::WindowInspected(observation)
        if matches!(observation.outcome, PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Indeterminate(_)))
        && observation.counters.acquired_bytes == target.range().length() as u64
        && observation.validation_counters.inspected_frames() == 0));
    assert!(
        matches!(handle.next_window(), Progress::DeadlineExceeded(counters) if counters.indeterminate_windows == 1)
    );
    assert_released(&serving);
    serving.close();
}
