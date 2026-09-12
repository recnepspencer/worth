use std::{
    path::Path,
    time::{Duration, Instant},
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, PhysicalSignalAspectBindingDigest,
    PhysicalSignalAspectBindingObservation, PhysicalWorkEffectFate, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition, RecordAppendBatch, RecordAppendDenial, RecordCountLimit,
    RecordReadDenial, RecordScanDenial, RecordScanOutcome, RecordScanRequest,
    ServingPhysicalRuntime,
};
use worth_store_physical_backend::{MediaCounterSnapshot, MediaFaultDirective, MediaOperationRole};

use super::super::durable_publication;
use super::{configuration, serving_from_initialization};

const PAYLOAD: &[u8] = b"scan continuation damage";
const SCAN_PARTITION: &str = "store.physical.record.scan";

#[derive(Debug, Clone, Copy)]
struct ScanCalibration {
    after_open: u64,
    after_scan_admission: u64,
}

#[test]
fn continuation_damage_revokes_shared_health_and_fences_mutation() {
    let calibration = calibrate();
    let (parent, serving) = open_faulted(calibration);
    let (before, invalidations_before, causal_start) =
        fail_first_continuation(&serving, calibration);
    await_signal_cleanup(&serving);
    assert_failed_effect(&serving, before, invalidations_before);
    assert_failed_scan_route(&serving, causal_start);
    assert_mutation_fenced(&serving);
    assert!(serving.close_plan().execute().requires_inspection());
    drop(parent);
}

fn open_faulted(calibration: ScanCalibration) -> (tempfile::TempDir, ServingPhysicalRuntime) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("faulted");
    seed(&root);
    let serving = super::fault_fixture::serving_from_open_with_identified_positioned_read_fault(
        &root,
        calibration.after_scan_admission + 1,
        MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    assert_eq!(
        identified_reads(&serving),
        calibration.after_open,
        "fresh open drifted from scan calibration"
    );
    (parent, serving)
}

fn fail_first_continuation(
    serving: &ServingPhysicalRuntime,
    calibration: ScanCalibration,
) -> (MediaCounterSnapshot, u64, usize) {
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
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
        RecordScanDenial::RecordRead(RecordReadDenial::ArtifactDamaged)
    );
    assert!(
        error.observation().physical_work_count() > 0,
        "scan denial must retain the failed canonical work identity"
    );
    drop(scan);
    (before, invalidations_before, causal_start)
}

fn assert_failed_effect(
    serving: &ServingPhysicalRuntime,
    before: MediaCounterSnapshot,
    invalidations_before: u64,
) {
    let role = MediaOperationRole::PositionedRead;
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

fn assert_failed_scan_route(serving: &ServingPhysicalRuntime, causal_start: usize) {
    let records = serving.physical_work_observer().causal().records();
    let failed = records[causal_start..]
        .iter()
        .filter(|record| {
            record.operation() == PhysicalWorkOperationFamily::ArtifactRangeRead
                && record.effect_fate() == PhysicalWorkEffectFate::ReadIncomplete
        })
        .collect::<Vec<_>>();
    assert_eq!(failed.len(), 1);
    assert!(failed[0].backend_operation().is_some());
    assert_eq!(
        failed[0].recovery(),
        PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert_eq!(
        partition_for(failed[0].signal_binding(), serving),
        SCAN_PARTITION
    );
}

fn assert_mutation_fenced(serving: &ServingPhysicalRuntime) {
    let (_, placement, _) = configuration();
    let before_blocked_append = serving.media_counters();
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.record_submission(),
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([215; 32]),
            RecordAppendBatch::try_from_iter([b"must remain unfired"]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert_eq!(
        serving.media_counters(),
        before_blocked_append,
        "shared health must fence mutation before media"
    );
}

fn calibrate() -> ScanCalibration {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("calibration");
    seed(&root);
    let serving = super::super::serving_from_open(&root);
    let after_open = identified_reads(&serving);
    let mut scan = serving
        .records()
        .expect("read protection admission")
        .scan(RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()))
        .unwrap();
    let after_scan_admission = identified_reads(&serving);
    let mut scratch = vec![0_u8; PAYLOAD.len()];
    assert!(matches!(
        scan.read_next_into(&mut scratch).unwrap(),
        RecordScanOutcome::Batch(_)
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

fn seed(root: &Path) {
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(root);
    durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("scan-continuation-damage", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    serving.close();
}

fn identified_reads(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .identified_operation_attempts_for(MediaOperationRole::PositionedRead)
}

fn signal(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::PhysicalSignalObservation {
    serving
        .physical_signal_observation()
        .expect("serving runtime owns Signal")
}

fn partition_for(
    digest: PhysicalSignalAspectBindingDigest,
    serving: &ServingPhysicalRuntime,
) -> String {
    serving
        .physical_signal_aspect_binding_observations()
        .iter()
        .find(|binding: &&PhysicalSignalAspectBindingObservation| binding.digest() == digest)
        .and_then(|binding| binding.partition())
        .expect("failed scan work must identify one installed partition")
        .partition
        .0
        .clone()
}

fn await_signal_cleanup(serving: &ServingPhysicalRuntime) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let observation = signal(serving);
        if observation.active_locality_count() == 0 && observation.active_in_flight_count() == 0 {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "damaged scan retained Signal state: locality={}, in_flight={}",
            observation.active_locality_count(),
            observation.active_in_flight_count(),
        );
        std::thread::yield_now();
    }
}
