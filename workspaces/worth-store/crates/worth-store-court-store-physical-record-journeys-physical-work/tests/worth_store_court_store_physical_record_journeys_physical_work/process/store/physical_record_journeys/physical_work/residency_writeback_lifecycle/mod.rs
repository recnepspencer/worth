#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::residency_writeback_lifecycle::*;
use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint,
    worth_store::physical_runtime::AdmittedDirtyFrame,
    worth_store::physical_runtime::PhysicalEffectObligation,
    worth_store::physical_runtime::PhysicalResidencyCertification,
    worth_store::physical_runtime::PhysicalSignalSettlementOutcome,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkPreEffectDenial,
    worth_store::physical_runtime::PhysicalWorkRecoveryDisposition,
    worth_store::physical_runtime::PhysicalWritebackExecution,
    worth_store::physical_runtime::PhysicalWritebackFailureCause,
};
use {
    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};
use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

#[test]
fn scheduler_admitted_cancellation_cannot_reach_writeback_dispatch() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let residency = serving.certification_physical_residency();
    let ready = residency
        .request_writeback(
            residency
                .prepare_writeback(
                    dirty_frame(&residency, writeback_coordinate()),
                    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                )
                .unwrap(),
        )
        .unwrap();
    let identity = ready.identity();
    let consumer = ready.consumer_handle();
    let signal_request = consumer.signal_request();
    let scheduler_before = serving.physical_scheduler_capacity();
    let admitted = residency.admit_writeback(ready).unwrap();
    let media_before = serving.media_counters();

    let cancellation = serving.cancel_physical_work(consumer).unwrap();

    assert_eq!(
        cancellation.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched,
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    assert_eq!(
        cancellation
            .signal()
            .cancelled_request()
            .map(|cancelled| cancelled.handle()),
        Some(signal_request),
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    let failure = match residency.execute_writeback(admitted) {
        Err(failure) => failure,
        Ok(_) => panic!("pre-dispatch cancelled writeback reached execution"),
    };
    assert_eq!(
        failure.cause(),
        worth_store::physical_runtime::PhysicalWritebackFailureCause::PreEffect(
            worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ConsumerCancelled
        ),
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    failure.into_dirty().discard().unwrap();
    let scheduler_after = serving.physical_scheduler_capacity();
    assert!(
        scheduler_after.configured() == scheduler_before.configured()
            && scheduler_after.available() == scheduler_before.available()
            && scheduler_after.active_reservations() == scheduler_before.active_reservations()
            && scheduler_after.admitted_reservations()
                == scheduler_before.admitted_reservations() + 1
            && scheduler_after.released_reservations()
                == scheduler_before.released_reservations() + 1
            && scheduler_after.denied_reservations() == scheduler_before.denied_reservations(),
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    assert_eq!(
        serving.media_counters(),
        media_before,
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    let closed = serving.close();
    assert_eq!(
        closed.work().drain().cancelled_before_dispatch(),
        &[identity],
        "C5_PREDICATE:predispatch-cancellation-dispatches"
    );
    assert!(closed
        .work()
        .drain()
        .continued_after_consumer_cancellation()
        .is_empty());
}

#[test]
fn cancellation_after_writeback_media_preserves_store_settlement() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let residency = serving.certification_physical_residency();
    let ready = residency
        .request_writeback(
            residency
                .prepare_writeback(
                    dirty_frame(&residency, writeback_coordinate()),
                    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                )
                .unwrap(),
        )
        .unwrap();
    let identity = ready.identity();
    let consumer = ready.consumer_handle();
    let signal_request = consumer.signal_request();
    let scheduler_before = serving.physical_scheduler_capacity();
    let admitted = residency.admit_writeback(ready).unwrap();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterResidencyWriteBeforeSchedulerSettlement,
    );
    let media_before = serving.media_counters();

    let execution = std::thread::scope(|scope| {
        let execution = scope.spawn(|| residency.execute_writeback(admitted));
        assert!(
            gate.await_arrival(),
            "the real writeback did not reach its post-media settlement checkpoint"
        );
        let cancellation = serving.cancel_physical_work(consumer).unwrap();
        assert_eq!(
            cancellation.obligation(),
            worth_store::physical_runtime::PhysicalEffectObligation::SettlementContinues
        );
        assert_eq!(
            cancellation
                .signal()
                .cancelled_request()
                .map(|cancelled| cancelled.handle()),
            Some(signal_request)
        );
        gate.release();
        execution.join().unwrap().unwrap()
    });

    let settlement = match execution {
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(settlement) => settlement,
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("exact writeback became retryable")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(_) => {
            panic!("exact writeback required inspection")
        }
    };
    assert_eq!(settlement.identity(), identity);
    assert!(settlement.effect().is_some());
    assert_eq!(
        settlement.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        settlement.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::ContinueSettlement
    );
    assert_eq!(
        settlement.signal(),
        worth_store::physical_runtime::PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        media_before
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            + 1
    );
    let scheduler_after = serving.physical_scheduler_capacity();
    assert!(
        scheduler_after.configured() == scheduler_before.configured()
            && scheduler_after.available() == scheduler_before.available()
            && scheduler_after.active_reservations() == scheduler_before.active_reservations()
            && scheduler_after.admitted_reservations()
                == scheduler_before.admitted_reservations() + 1
            && scheduler_after.released_reservations()
                == scheduler_before.released_reservations() + 1
            && scheduler_after.denied_reservations() == scheduler_before.denied_reservations(),
        "C5_PREDICATE:postdispatch-cancellation-loses-terminal-fate"
    );
    let closed = serving.close();
    assert_eq!(
        closed
            .work()
            .drain()
            .continued_after_consumer_cancellation(),
        &[identity],
        "C5_PREDICATE:postdispatch-cancellation-loses-terminal-fate"
    );
}

#[test]
fn predispatch_timeout_returns_dirty_authority_without_a_media_attempt() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_work_profile(root.path(), profile);
    let residency = serving.certification_physical_residency();
    let coordinate = writeback_coordinate();
    let ready = residency
        .request_writeback(
            residency
                .prepare_writeback(
                    dirty_frame(&residency, coordinate),
                    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                )
                .unwrap(),
        )
        .unwrap();
    let consumer = ready.consumer_handle();
    let media_before = serving.media_counters();
    let scheduler_before = serving.physical_scheduler_capacity();
    let writeback_before = serving.residency_observation().writebacks();
    serving
        .advance_physical_signal_clock(consumer, clock_advance(1_000))
        .unwrap();

    let timeout = serving.timeout_physical_work(consumer).unwrap();

    assert_eq!(
        timeout.obligation(),
        worth_store::physical_runtime::PhysicalEffectObligation::NotDispatched
    );
    let failure = match residency.admit_writeback(ready) {
        Err(failure) => failure,
        Ok(_) => panic!("a timed-out writeback must not enter scheduler admission"),
    };
    assert!(matches!(
        failure.cause(),
        worth_store::physical_runtime::PhysicalWritebackFailureCause::PreEffect(
            worth_store::physical_runtime::PhysicalWorkPreEffectDenial::ConsumerCancelled
        )
    ));
    let dirty = failure.into_dirty();
    assert_eq!(dirty.coordinate(), coordinate);
    assert_eq!(residency.counters().dirty_frames(), 1);
    assert_eq!(serving.media_counters(), media_before);
    assert_eq!(serving.physical_scheduler_capacity(), scheduler_before);
    assert_eq!(
        serving.residency_observation().writebacks(),
        writeback_before
    );
    dirty.discard().unwrap();
    assert!(!serving.close().residency().requires_inspection());
}

#[test]
fn partial_writeback_retains_dirty_truth_and_exact_indeterminate_observation() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let coordinate = writeback_coordinate();
    let before_bytes = artifact_range(root.path(), coordinate);
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let residency = serving.certification_physical_residency();
    let writeback_before = serving.residency_observation().writebacks();
    let admitted = residency
        .admit_writeback(
            residency
                .request_writeback(
                    residency
                        .prepare_writeback(
                            dirty_frame(&residency, coordinate),
                            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                        )
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
    let media_before = serving.media_counters();
    let writeback_after_admission = serving.residency_observation().writebacks();
    assert_eq!(
        writeback_after_admission.attempts(),
        writeback_before.attempts() + 1
    );

    let inspection = match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(
            inspection,
        ) => inspection,
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(_) => {
            panic!("a partial physical writeback cannot publish clean")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("a partial physical writeback is not safe to retry")
        }
    };

    let settlement = inspection.settlement();
    assert_eq!(
        settlement.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::Indeterminate
    );
    assert_eq!(
        settlement.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert_eq!(residency.counters().dirty_frames(), 1);
    let observed = artifact_range(root.path(), coordinate);
    assert_eq!(&observed[..3], &WRITEBACK[..3]);
    assert_eq!(&observed[3..], &before_bytes[3..]);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        media_before
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            + 1
    );
    let writeback_after = serving.residency_observation().writebacks();
    assert_eq!(
        writeback_after.attempts(),
        writeback_after_admission.attempts()
    );
    assert_eq!(
        writeback_after.indeterminate(),
        writeback_before.indeterminate() + 1
    );
    assert_eq!(
        writeback_after.inspection_required(),
        writeback_before.inspection_required() + 1
    );
    assert_eq!(
        writeback_after.exact_receipts(),
        writeback_before.exact_receipts()
    );
    assert_eq!(writeback_after.retryable(), writeback_before.retryable());
    assert!(serving.close().residency().requires_inspection());
}
