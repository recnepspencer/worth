use super::pressure_world::{read_record, snapshot, PressureWorld, PAGE_BYTES, RESIDENT_BYTES};
use super::{assert_released, Progress, Request, Target};
use std::time::Duration;
use worth_store::physical_runtime::{
    certification::CertificationPhysicalExecutionCheckpoint as Checkpoint,
    PhysicalIntegrityScrubDeferral, PhysicalOperationAllocationScope, ServingPhysicalRuntime,
};
use worth_store_io_scheduler::foreground_reservation::{
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, ForegroundResourceBudget, QueueSlot,
};
use worth_store_physical_integrity::PhysicalIntegrityObservationOutcome;

#[test]
fn bounded_scrub_preserves_foreground_progress_at_thirty_two_times_residency() {
    let parent = tempfile::tempdir().unwrap();
    let PressureWorld {
        serving,
        targets,
        records,
    } = PressureWorld::new(parent.path());
    let unchanged = snapshot(parent.path());
    let request = || {
        Request::new(
            serving.store_identity(),
            targets.iter().copied(),
            PAGE_BYTES,
            targets.len() as u64 * u64::from(PAGE_BYTES),
            Duration::from_secs(60),
        )
        .unwrap()
    };
    let mut cancelled = serving.start_physical_integrity_scrub(request()).unwrap();
    cancelled.cancellation().cancel();
    assert!(matches!(cancelled.next_window(), Progress::Cancelled(c) if c.acquired_bytes == 0));
    let mut scrub = serving.start_physical_integrity_scrub(request()).unwrap();

    // A real foreground reservation consumes the background share, while leaving
    // capacity for ordinary reads. This is not a forced scrub return value.
    let capacity = serving.physical_scheduler_capacity();
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "scrub-pressure",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(
                    QueueSlot::new(capacity.configured().queue_slots() / 2 + 1).unwrap(),
                )
                .with_worker_permits(worth_store_io_scheduler::WorkerPermit::new(1).unwrap()),
        );
    let held = serving.reserve_physical_scheduler_foreground(lane).unwrap();
    let before = serving.media_counters();
    assert!(matches!(
        scrub.next_window(),
        Progress::Deferred(PhysicalIntegrityScrubDeferral::SchedulerOrDependency(
            worth_store::physical_runtime::PhysicalIntegrityScrubReadDeferral::Capacity(_)
        ))
    ));
    assert_eq!(
        serving.media_counters(),
        before,
        "denial must precede backend access"
    );
    assert_eq!(scrub.counters().acquired_bytes, 0);
    assert_eq!(scrub.counters().completed_windows, 0);
    assert_eq!(
        serving.physical_scheduler_capacity().denied_reservations(),
        capacity.denied_reservations() + 1
    );
    assert_eq!(
        serving
            .physical_work_counters()
            .total(worth_store::physical_runtime::PhysicalWorkCounterStage::Ready),
        0,
        "a deferred diagnostic window must not retain submitted ready work"
    );
    let signal = serving.physical_signal_observation().unwrap();
    assert_eq!(signal.active_in_flight_count(), 0);
    assert_eq!(signal.active_locality_count(), 0);
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Scrub),
        0
    );
    read_record(&serving, &records, 0);
    drop(held);
    assert_released(&serving);

    let resume = scrub.pause();
    assert!(matches!(scrub.next_window(), Progress::Paused));
    scrub.resume(resume).unwrap();
    let before_pressure = serving.residency_observation().counters();
    for (ordinal, target) in targets.iter().enumerate() {
        let Progress::WindowInspected(window) = scrub.next_window() else {
            panic!("declared page window");
        };
        assert_eq!(window.ordinal, ordinal as u64);
        assert!(
            matches!(window.outcome, PhysicalIntegrityObservationOutcome::Intact(scope) if scope == target.scope())
        );
        assert_eq!(window.validation_counters.inspected_frames(), 1);
        assert_eq!(
            window.counters.acquired_bytes,
            (ordinal as u64 + 1) * u64::from(PAGE_BYTES)
        );
        assert_eq!(window.counters.peak_allocation_bytes, u64::from(PAGE_BYTES));
        assert_released(&serving);
        read_record(&serving, &records, (ordinal * 2).min(records.len() - 1));
        if ordinal == targets.len() / 2 {
            let evicted = serving.residency_observation().counters();
            assert!(evicted.evictions() > before_pressure.evictions());
            read_record(&serving, &records, 0);
            assert!(
                serving.residency_observation().counters().source_loads() > evicted.source_loads(),
                "ordinary record must refault between scrub windows"
            );
            assert!(ordinal + 1 < targets.len());
        }
        assert!(
            serving
                .residency_observation()
                .counters()
                .peak_resident_bytes()
                <= RESIDENT_BYTES
        );
    }
    assert!(
        matches!(scrub.next_window(), Progress::Completed(c) if c.completed_windows == targets.len() as u64 && c.deferred_windows == 1)
    );
    assert_eq!(
        snapshot(parent.path()),
        unchanged,
        "scrub and reads preserve protected bytes and all paths; OS lease payload is excluded"
    );
    cancel_after_read(&serving, targets[0]);
    assert_eq!(snapshot(parent.path()), unchanged);
    close_with_outstanding_window(serving, targets[0]);
}

fn single_request(serving: &ServingPhysicalRuntime, target: Target) -> Request {
    Request::new(
        serving.store_identity(),
        [target],
        PAGE_BYTES,
        u64::from(PAGE_BYTES),
        Duration::from_secs(30),
    )
    .unwrap()
}

fn cancel_after_read(serving: &ServingPhysicalRuntime, target: Target) {
    let mut scrub = serving
        .start_physical_integrity_scrub(single_request(serving, target))
        .unwrap();
    let cancellation = scrub.cancellation();
    let gate = serving
        .certification_pause_physical_execution_at(Checkpoint::AfterReadBeforeSchedulerSettlement);
    let worker = std::thread::spawn(move || {
        let progress = scrub.next_window();
        (scrub, progress)
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
        u64::from(PAGE_BYTES)
    );
    cancellation.cancel();
    gate.release();
    let (mut scrub, progress) = worker.join().unwrap();
    assert!(
        matches!(progress, Progress::WindowInspected(w) if w.counters.acquired_bytes == u64::from(PAGE_BYTES))
    );
    assert!(matches!(scrub.next_window(), Progress::Cancelled(c) if c.completed_windows == 1));
    assert_released(serving);
}

fn close_with_outstanding_window(serving: ServingPhysicalRuntime, target: Target) {
    let mut scrub = serving
        .start_physical_integrity_scrub(single_request(&serving, target))
        .unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(Checkpoint::AfterReadBeforeSchedulerSettlement);
    let reader = std::thread::spawn(move || {
        let result = scrub.next_window();
        (scrub, result)
    });
    assert!(gate.await_arrival());
    let (tx, rx) = std::sync::mpsc::channel();
    let closer = std::thread::spawn(move || {
        let closed = serving.close();
        tx.send(closed).unwrap();
    });
    assert!(rx.recv_timeout(Duration::from_millis(50)).is_err());
    gate.release();
    let (mut scrub, progress) = reader.join().unwrap();
    closer.join().unwrap();
    let closed = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!closed.residency().requires_inspection());
    assert!(
        matches!(progress, Progress::WindowInspected(w) if w.counters.acquired_bytes == u64::from(PAGE_BYTES))
    );
    assert!(matches!(scrub.next_window(), Progress::Closed(c) if c.completed_windows == 1));
}
