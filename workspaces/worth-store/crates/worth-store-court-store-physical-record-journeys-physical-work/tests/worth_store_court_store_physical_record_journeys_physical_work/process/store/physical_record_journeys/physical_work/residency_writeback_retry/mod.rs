#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::residency_writeback_retry::*;
use {worth_store::physical_runtime::AdmittedDirtyFrame, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkRecoveryDisposition, worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome, worth_store::physical_runtime::PhysicalWritebackExecution};
use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};
use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

use {worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

#[test]
fn no_effect_writeback_retains_dirty_ownership_through_signal_retry() {
    let root = tempfile::tempdir().unwrap();
    let (profile, _, _) = work_fixture();
    serving_from_initialization_with_work_profile(root.path(), profile.clone()).close();
    let serving = serving_from_open_with_positioned_write_fault(
        root.path(),
        profile,
        worth_store_physical_backend::MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let residency = serving.certification_physical_residency();
    let coordinate =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap();
    let before_bytes = artifact_range(root.path(), coordinate);
    let writeback_before = serving.residency_observation().writebacks();
    let dirty = dirty_frame(&residency, coordinate);
    let ready = residency
        .request_writeback(
            residency
                .prepare_writeback(
                    dirty,
                    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                )
                .unwrap(),
        )
        .unwrap();
    let consumer = ready.consumer_handle();
    let identity = ready.identity();
    let admitted = residency.admit_writeback(ready).unwrap();
    let before = serving.media_counters();
    let writeback_after_admission = serving.residency_observation().writebacks();
    assert_eq!(
        writeback_after_admission.attempts(),
        writeback_before.attempts() + 1
    );

    let retryable = match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(retryable) => retryable,
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(_) => {
            panic!("faulted residency writeback must retain a safe retry")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(_) => {
            panic!("a proven no-effect writeback must not require inspection")
        }
    };

    assert_eq!(retryable.settled().intent().identity(), identity);
    assert_eq!(
        retryable.settled().evidence().fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ProvenNoEffect
    );
    assert_eq!(
        retryable.settled().recovery_disposition(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::RetryExact
    );
    assert_eq!(artifact_range(root.path(), coordinate), before_bytes);
    assert_eq!(residency.counters().dirty_frames(), 1);
    let retry_observation = serving.residency_observation().writebacks();
    assert_eq!(
        retry_observation.attempts(),
        writeback_after_admission.attempts()
    );
    assert_eq!(
        retry_observation.retryable(),
        writeback_before.retryable() + 1
    );
    assert_eq!(
        retry_observation.exact_receipts(),
        writeback_before.exact_receipts()
    );
    assert_eq!(
        completed_writes(&serving)
            - before.completed_operations_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        0
    );
    assert_eq!(
        denied_writes(&serving)
            - before.denied_before_effect_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        1
    );

    serving
        .advance_physical_signal_clock(consumer, clock_advance(1_000))
        .unwrap();
    serving.timeout_physical_work(consumer).unwrap();
    let retry = match serving
        .schedule_physical_work_retry(retryable.settled())
        .unwrap()
    {
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Scheduled(retry) => retry,
        worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome::Denied(report) => {
            panic!("no-effect residency retry should schedule: {report:?}")
        }
    };
    serving
        .advance_physical_signal_clock(consumer, clock_advance(1_001))
        .unwrap();
    let (settled, dirty) = retryable.into_parts();
    let retry = serving.admit_physical_work_retry(&retry, settled).unwrap();
    let retry_consumer = retry
        .consumer_handle()
        .expect("Signal admitted the residency retry generation");
    assert_ne!(retry_consumer.signal_request(), consumer.signal_request());
    let (ready, retry_command, _) = retry.into_parts();
    assert_eq!(ready.intent().identity(), identity);
    let ready = residency.bind_writeback_retry(ready, dirty).unwrap();
    assert_eq!(
        serving.residency_observation().writebacks().attempts(),
        writeback_before.attempts() + 2
    );
    let admitted = residency
        .admit_writeback_retry(ready, retry_command)
        .unwrap_or_else(|failure| {
            panic!(
                "C5_PREDICATE:local-physical-work-scheduler: canonical retry identity was rejected by local scheduler state: {:?}",
                failure.cause()
            )
        });
    let settlement = match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(settlement) => settlement,
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("unfaulted residency retry must settle clean")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(_) => {
            panic!("unfaulted residency retry must not require inspection")
        }
    };

    assert_eq!(settlement.identity(), identity);
    assert_eq!(
        settlement.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted
    );
    assert_eq!(
        completed_writes(&serving),
        before.completed_operations_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite) + 1
    );
    assert_eq!(
        denied_writes(&serving),
        before.denied_before_effect_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite) + 1
    );
    assert_eq!(artifact_range(root.path(), coordinate), EXPECTED_WRITEBACK);
    assert_eq!(residency.counters().dirty_frames(), 0);
    let final_observation = serving.residency_observation().writebacks();
    assert_eq!(
        final_observation.attempts(),
        writeback_before.attempts() + 2
    );
    assert_eq!(
        final_observation.exact_receipts(),
        writeback_before.exact_receipts() + 1
    );
    assert_eq!(
        final_observation.retryable(),
        writeback_before.retryable() + 1
    );
    assert_eq!(
        final_observation.indeterminate(),
        writeback_before.indeterminate()
    );
    assert_eq!(
        final_observation.inspection_required(),
        writeback_before.inspection_required()
    );
    assert!(!serving.close().residency().requires_inspection());
}
