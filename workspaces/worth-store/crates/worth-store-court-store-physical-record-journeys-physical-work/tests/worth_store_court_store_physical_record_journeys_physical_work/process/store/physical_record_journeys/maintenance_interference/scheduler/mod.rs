#[allow(unused_imports)] use worth_store_testkit_store::physical_record_journeys::maintenance_interference::scheduler::*;
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalReadWorkRequest, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PhysicalWorkCapacityDimension, worth_store::physical_runtime::PhysicalWorkReadiness, worth_store::physical_runtime::RecordSchedulerReservationDenial, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial, worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

#[test]
fn canonical_profile_keeps_eight_ready_slots_and_two_background_dispatches() {
    let parent = tempfile::tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(8, 1, 1_024, 1024 * 1024, 4 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let serving = serving_from_initialization_with_work_profile(
        &parent.path().join("store"),
        profile.with_capacity(capacity),
    );
    let configured = serving.physical_scheduler_capacity().configured();
    assert_eq!(configured.queue_slots(), 8);
    assert_eq!(configured.worker_permits(), 4);

    let submission = serving.physical_read_submission();
    for _ in 0..8 {
        match submission.submit(request.clone()).into_raw() {
            worth_proof::TransitionOutcome::Success(_) => {}
            other => panic!("eight ready slots must admit, saw {other:?}"),
        }
    }
    let before = serving.media_counters();
    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Deferred(deferred)
            if deferred.capacity() == 8
                && deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::Commands
    ));
    assert_eq!(serving.media_counters(), before);

    let mut foreground = Vec::new();
    for _ in 0..4 {
        foreground.push(
            serving
                .reserve_physical_scheduler_foreground(dispatch_lane())
                .expect("four idle dispatch permits must admit"),
        );
    }
    let before = serving.media_counters();
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(dispatch_lane()),
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall::WorkerPermit {
                        requested: 1,
                        available: 0,
                    },
                ),
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    drop(foreground);

    let mut background = Vec::new();
    for _ in 0..2 {
        background.push(
            serving
                .reserve_physical_scheduler_background(dispatch_lane())
                .expect("two background dispatches fit beside the foreground floor"),
        );
    }
    let before = serving.media_counters();
    assert!(matches!(
        serving.reserve_physical_scheduler_background(dispatch_lane()),
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall::WorkerPermit { requested: 1, .. },
                ),
            ),
        ))
    ));
    let mut protected = Vec::new();
    for _ in 0..2 {
        protected.push(
            serving
                .reserve_physical_scheduler_foreground(dispatch_lane())
                .expect("two dispatch permits stay available to foreground"),
        );
    }
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(dispatch_lane()),
        Err(worth_store::physical_runtime::RecordSchedulerReservationDenial::Admission(
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial::Foreground(
                worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial::InsufficientCapacity(
                    worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall::WorkerPermit {
                        requested: 1,
                        available: 0,
                    },
                ),
            ),
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    drop(protected);
    drop(background);
    serving.close();
}
