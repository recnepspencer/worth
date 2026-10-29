pub(crate) use {std::path::Path, std::time::Duration, std::time::Instant};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalSignalAspectBindingDigest, worth_store::physical_runtime::PhysicalSignalAspectBindingObservation, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkRecoveryDisposition, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial, worth_store::physical_runtime::RecordCountLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordScanDenial, worth_store::physical_runtime::RecordScanOutcome, worth_store::physical_runtime::RecordScanRequest, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_backend::MediaCounterSnapshot, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::durable_publication};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) const PAYLOAD: &[u8] = b"scan continuation damage";
pub(crate) const SCAN_PARTITION: &str = "store.physical.record.scan";

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScanCalibration {
    pub(crate) after_open: u64,
    pub(crate) after_scan_admission: u64,
}

pub(crate) fn open_faulted(calibration: ScanCalibration) -> (tempfile::TempDir, ServingPhysicalRuntime) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("faulted");
    seed(&root);
    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_identified_positioned_read_fault(
        &root,
        calibration.after_scan_admission + 1,
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    assert_eq!(
        identified_reads(&serving),
        calibration.after_open,
        "fresh open drifted from scan calibration"
    );
    (parent, serving)
}

pub(crate) fn fail_first_continuation(
    serving: &ServingPhysicalRuntime,
    calibration: ScanCalibration,
) -> (MediaCounterSnapshot, u64, usize) {
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
        .expect("the fault must occur after scan admission");
    assert_eq!(
        identified_reads(serving),
        calibration.after_scan_admission,
        "scan admission crossed the scheduled continuation fault"
    );
    let before = serving.media_counters();
    let invalidations_before = signal(serving).aspect_invalidation_count();
    let causal_start = serving.physical_work_observer().causal().records().len();
    let mut scratch = vec![0_u8; PAYLOAD.len()];
    let error = match scan.read_next_into(&mut scratch) {
        Err(error) => error,
        Ok(_) => panic!("the first continuation read must encounter the fault"),
    };
    assert_eq!(
        error.denial(),
        worth_store::physical_runtime::RecordScanDenial::RecordRead(worth_store::physical_runtime::RecordReadDenial::ArtifactDamaged)
    );
    assert!(
        error.observation().physical_work_count() > 0,
        "scan denial must retain the failed canonical work identity"
    );
    drop(scan);
    (before, invalidations_before, causal_start)
}

pub(crate) fn assert_failed_effect(
    serving: &ServingPhysicalRuntime,
    before: MediaCounterSnapshot,
    invalidations_before: u64,
) {
    let role = worth_store_physical_backend::MediaOperationRole::PositionedRead;
    let after = serving.media_counters();
    assert_eq!(after.fault_matches(), before.fault_matches() + 1);
    assert_eq!(
        after.identified_operation_attempts_for(role)
            - before.identified_operation_attempts_for(role),
        1
    );
    assert_eq!(after.attempts_for(role) - before.attempts_for(role), 1);
    assert_eq!(
        after.denied_before_effect_for(role) - before.denied_before_effect_for(role),
        0
    );
    assert_eq!(
        after.completed_operations_for(role) - before.completed_operations_for(role),
        0,
        "a retained three-byte prefix is a real effect, not a completed exact read"
    );
    assert_eq!(
        after.completed_bytes_for(role) - before.completed_bytes_for(role),
        3
    );
    assert_eq!(
        signal(serving).aspect_invalidation_count(),
        invalidations_before + 1
    );
}

pub(crate) fn assert_failed_scan_route(serving: &ServingPhysicalRuntime, causal_start: usize) {
    let records = serving.physical_work_observer().causal().records();
    let failed = records[causal_start..]
        .iter()
        .filter(|record| {
            record.operation() == worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
                && record.effect_fate() == worth_store::physical_runtime::PhysicalWorkEffectFate::ReadIncomplete
        })
        .collect::<Vec<_>>();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].backend_operation().is_some());
    assert_eq!(
        failed[0].recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert_eq!(
        partition_for(failed[0].signal_binding(), serving),
        SCAN_PARTITION
    );
}

pub(crate) fn assert_mutation_fenced(serving: &ServingPhysicalRuntime) {
    let (_, placement, _) = configuration();
    let before_blocked_append = serving.media_counters();
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([215; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"must remain unfired"]).unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
            worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert_eq!(
        serving.media_counters(),
        before_blocked_append,
        "shared health must fence mutation before media"
    );
}

pub(crate) fn calibrate() -> ScanCalibration {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("calibration");
    seed(&root);
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    let after_open = identified_reads(&serving);
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let after_scan_admission = identified_reads(&serving);
    let mut scratch = vec![0_u8; PAYLOAD.len()];
    assert!(matches!(
        scan.read_next_into(&mut scratch).unwrap(),
        worth_store::physical_runtime::RecordScanOutcome::Batch(_)
    ));
    assert!(
        identified_reads(&serving) > after_scan_admission,
        "scan continuation must cross at least one positioned-read boundary"
    );
    drop(scan);
    await_signal_cleanup(&serving);
    assert!(!serving.close_plan().execute().requires_inspection());
    ScanCalibration {
        after_open,
        after_scan_admission,
    }
}

pub(crate) fn seed(root: &Path) {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material("scan-continuation-damage", 1),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    serving.close();
}

pub(crate) fn identified_reads(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .identified_operation_attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
}

pub(crate) fn signal(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::PhysicalSignalObservation {
    serving
        .physical_signal_observation()
        .expect("serving runtime owns Signal")
}

pub(crate) fn partition_for(
    digest: PhysicalSignalAspectBindingDigest,
    serving: &ServingPhysicalRuntime,
) -> String {
    serving
        .physical_signal_aspect_binding_observations()
        .iter()
        .find(|binding: &&PhysicalSignalAspectBindingObservation| binding.digest() == digest)
        .and_then(|binding| binding.partition())
        .expect("failed scan work must identify one installed partition")
        .path()
        .segments()[0]
        .clone()
}

pub(crate) fn await_signal_cleanup(serving: &ServingPhysicalRuntime) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let observation = signal(serving);
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "damaged scan retained Signal state: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}