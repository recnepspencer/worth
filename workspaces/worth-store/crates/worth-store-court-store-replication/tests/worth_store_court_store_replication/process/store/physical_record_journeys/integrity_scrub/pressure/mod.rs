#[allow(unused_imports)] use crate::physical_record_journeys::integrity_scrub::pressure::*;
use {crate::physical_record_journeys::integrity_scrub::pressure_world::read_record, crate::physical_record_journeys::integrity_scrub::pressure_world::snapshot, crate::physical_record_journeys::integrity_scrub::pressure_world::PressureWorld, crate::physical_record_journeys::integrity_scrub::pressure_world::PAGE_BYTES, crate::physical_record_journeys::integrity_scrub::pressure_world::RESIDENT_BYTES};
use {worth_store_testkit_store::integrity_scrub::assert_released, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest as Request, worth_store::physical_runtime::PhysicalIntegrityScrubTarget as Target};
use {std::time::Duration};
use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint as Checkpoint, worth_store::physical_runtime::PhysicalIntegrityScrubDeferral, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot};
use {worth_store_physical_integrity::PhysicalIntegrityObservationOutcome};

#[test]
fn bounded_scrub_preserves_foreground_progress_at_thirty_two_times_residency() {
    let parent = tempfile::tempdir().unwrap();
    let PressureWorld {
        serving,
        targets,
        records,
    } = crate::physical_record_journeys::integrity_scrub::pressure_world::PressureWorld::new(parent.path());
    let unchanged = snapshot(parent.path());
    let request = || {
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
            serving.store_identity(),
            targets.iter().copied(),
            PAGE_BYTES,
            targets.len() as u64 * u64::from(PAGE_BYTES),
            std::time::Duration::from_secs(60),
        )
        .unwrap()
    };
    let mut cancelled = serving.start_physical_integrity_scrub(request()).unwrap();
    cancelled.cancellation().cancel();
    assert!(matches!(cancelled.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(c) if c.acquired_bytes == 0));
    let mut scrub = serving.start_physical_integrity_scrub(request()).unwrap();

    // A real foreground reservation consumes the background share, while leaving
    // capacity for ordinary reads. This is not a forced scrub return value.
    let capacity = serving.physical_scheduler_capacity();
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "scrub-pressure",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(
                    worth_store_io_scheduler::foreground_reservation::QueueSlot::new(capacity.configured().queue_slots() / 2 + 1).unwrap(),
                )
                .with_worker_permits(worth_store_io_scheduler::WorkerPermit::new(1).unwrap()),
        );
    let held = serving.reserve_physical_scheduler_foreground(lane).unwrap();
    let before = serving.media_counters();
    assert!(matches!(
        scrub.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Deferred(worth_store::physical_runtime::PhysicalIntegrityScrubDeferral::SchedulerOrDependency(
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
            .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub),
        0
    );
    let ordinary = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "scrub-pressure-foreground-floor",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::WorkerPermit::new(1).unwrap()),
        );
    let ordinary = serving
        .reserve_physical_scheduler_foreground(ordinary)
        .expect("foreground floor remains after a capacity-deferred scrub");
    drop(ordinary);
    drop(held);
    assert_released(&serving);

    let resume = scrub.pause();
    assert!(matches!(scrub.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Paused));
    scrub.resume(resume).unwrap();
    let before_pressure = serving.residency_observation().counters();
    for (ordinal, target) in targets.iter().enumerate() {
        let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(window) = scrub.next_window() else {
            panic!("declared page window");
        };
        assert_eq!(window.ordinal, ordinal as u64);
        assert!(
            matches!(window.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Intact(scope) if scope == target.scope())
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
        matches!(scrub.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Completed(c) if c.completed_windows == targets.len() as u64 && c.deferred_windows == 1)
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
