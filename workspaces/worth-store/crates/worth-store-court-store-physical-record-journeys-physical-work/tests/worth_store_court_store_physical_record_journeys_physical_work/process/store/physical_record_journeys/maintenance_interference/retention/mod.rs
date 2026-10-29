#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::retention::*;
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalReadProtectionPolicy, worth_store::physical_runtime::PhysicalRecordFormatDeclaration, worth_store::physical_runtime::PhysicalRecordId, worth_store::physical_runtime::RecordReadDenial};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize_with_wal, worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement};

#[test]
fn pinned_reader_rejects_one_over_growth_then_retries_after_reclaim() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let serving = initialize_with_wal(
        &root,
        format,
        64 * 1024,
        worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
        64 * 1024,
    );
    const GROWTH: u64 = 1024 * 1024;
    serving.certification_limit_candidate_growth_bytes(GROWTH);
    let policy = placement();
    let baseline = expect_published(&serving, policy, 1, b"pinned-baseline");
    let reader = serving.records().unwrap();
    let mut published = 1_u64;
    loop {
        published += 1;
        assert!(published < 80, "growth never denied a later append");
        match attempt(&serving, policy, published, b"fill") {
            crate::physical_record_journeys::maintenance_interference::retention::Attempt::Published(_) => {}
            crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::RetentionPressure) => {
                break;
            }
            crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(cause) => panic!("one-over growth denied as {cause:?}"),
        }
    }
    let writes = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let charged = serving.certification_charged_growth_bytes();
    assert!(charged <= GROWTH, "charged {charged} exceeded {GROWTH}");
    match attempt(&serving, policy, published + 1, b"still-over") {
        crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::RetentionPressure) => {}
        other => panic!("the same ceiling must deny again: {other:?}"),
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes
    );
    let mut held = reader.open(baseline, limits()).unwrap();
    assert_eq!(
        held.next_chunk().unwrap().unwrap().bytes(),
        b"pinned-baseline"
    );
    drop(held);
    let rotations = serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0);
    assert!(rotations >= 1, "reclaim needs a completed WAL segment");
    checkpoint(&serving, 90_000);
    let retried = match attempt(&serving, policy, published + 2, b"after-reclaim") {
        crate::physical_record_journeys::maintenance_interference::retention::Attempt::Published(id) => id,
        crate::physical_record_journeys::maintenance_interference::retention::Attempt::NoEffect(cause) => panic!(
            "reclaim must admit one more append under the same ceiling: {cause:?} charged={}",
            serving.certification_charged_growth_bytes()
        ),
    };
    assert!(serving.certification_charged_growth_bytes() <= GROWTH);
    assert!(matches!(
        reader.open(retried, limits()),
        Err(error) if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound
    ));
    let mut still = reader.open(baseline, limits()).unwrap();
    assert_eq!(
        still.next_chunk().unwrap().unwrap().bytes(),
        b"pinned-baseline"
    );
    drop(still);
    drop(reader);
    serving.close();
}
