pub(crate) use std::time::Duration;
pub(crate) use worth_store_physical_integrity::PhysicalIntegrityObservationOutcome;
pub(crate) use {
    crate::physical_record_journeys::integrity_scrub::pressure_world::read_record,
    crate::physical_record_journeys::integrity_scrub::pressure_world::snapshot,
    crate::physical_record_journeys::integrity_scrub::pressure_world::PressureWorld,
    crate::physical_record_journeys::integrity_scrub::pressure_world::PAGE_BYTES,
    crate::physical_record_journeys::integrity_scrub::pressure_world::RESIDENT_BYTES,
};
pub(crate) use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint as Checkpoint,
    worth_store::physical_runtime::PhysicalIntegrityScrubDeferral,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest as Request,
    worth_store::physical_runtime::PhysicalIntegrityScrubTarget as Target,
    worth_store_testkit_store::integrity_scrub::assert_released,
};
pub(crate) use {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
    worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope,
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
    worth_store_io_scheduler::foreground_reservation::QueueSlot,
};

pub(crate) fn single_request(serving: &ServingPhysicalRuntime, target: Target) -> Request {
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        PAGE_BYTES,
        u64::from(PAGE_BYTES),
        std::time::Duration::from_secs(30),
    )
    .unwrap()
}

pub(crate) fn cancel_after_read(serving: &ServingPhysicalRuntime, target: Target) {
    let mut scrub = serving
        .start_physical_integrity_scrub(single_request(serving, target))
        .unwrap();
    let cancellation = scrub.cancellation();
    let gate = serving
        .certification_pause_physical_execution_at(worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement);
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
            .active_operation_bytes_for(
                worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub
            ),
        u64::from(PAGE_BYTES)
    );
    cancellation.cancel();
    gate.release();
    let (mut scrub, progress) = worker.join().unwrap();
    assert!(
        matches!(progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(w) if w.counters.acquired_bytes == u64::from(PAGE_BYTES))
    );
    assert!(
        matches!(scrub.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(c) if c.completed_windows == 1)
    );
    assert_released(serving);
}

pub(crate) fn close_with_outstanding_window(serving: ServingPhysicalRuntime, target: Target) {
    let mut scrub = serving
        .start_physical_integrity_scrub(single_request(&serving, target))
        .unwrap();
    let gate = serving
        .certification_pause_physical_execution_at(worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement);
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
    assert!(rx
        .recv_timeout(std::time::Duration::from_millis(50))
        .is_err());
    gate.release();
    let (mut scrub, progress) = reader.join().unwrap();
    closer.join().unwrap();
    let closed = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(!closed.residency().requires_inspection());
    assert!(
        matches!(progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(w) if w.counters.acquired_bytes == u64::from(PAGE_BYTES))
    );
    assert!(
        matches!(scrub.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Closed(c) if c.completed_windows == 1)
    );
}
