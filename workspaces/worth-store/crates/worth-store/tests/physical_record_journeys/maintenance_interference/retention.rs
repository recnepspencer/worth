use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationRequest, PhysicalReadProtectionPolicy,
    PhysicalRecordFormatDeclaration, PhysicalRecordId, RecordReadDenial,
};
use worth_store_physical_backend::MediaOperationRole;

use super::{checkpoint, initialize_with_wal, limits, placement};

#[test]
fn pinned_reader_rejects_one_over_growth_then_retries_after_reclaim() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let serving = initialize_with_wal(
        &root,
        format,
        64 * 1024,
        PhysicalReadProtectionPolicy::default(),
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
            Attempt::Published(_) => {}
            Attempt::NoEffect(PhysicalMutationProvenNoEffectCause::RetentionPressure) => {
                break;
            }
            Attempt::NoEffect(cause) => panic!("one-over growth denied as {cause:?}"),
        }
    }
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let charged = serving.certification_charged_growth_bytes();
    assert!(charged <= GROWTH, "charged {charged} exceeded {GROWTH}");
    match attempt(&serving, policy, published + 1, b"still-over") {
        Attempt::NoEffect(PhysicalMutationProvenNoEffectCause::RetentionPressure) => {}
        other => panic!("the same ceiling must deny again: {other:?}"),
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
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
        Attempt::Published(id) => id,
        Attempt::NoEffect(cause) => panic!(
            "reclaim must admit one more append under the same ceiling: {cause:?} charged={}",
            serving.certification_charged_growth_bytes()
        ),
    };
    assert!(serving.certification_charged_growth_bytes() <= GROWTH);
    assert!(matches!(
        reader.open(retried, limits()),
        Err(error) if error.denial() == RecordReadDenial::RecordNotFound
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

#[derive(Debug)]
enum Attempt {
    Published(PhysicalRecordId),
    NoEffect(PhysicalMutationProvenNoEffectCause),
}

fn expect_published(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> PhysicalRecordId {
    match attempt(serving, policy, ordinal, bytes) {
        Attempt::Published(id) => id,
        Attempt::NoEffect(cause) => panic!("append {ordinal} had no effect: {cause:?}"),
    }
}

fn attempt(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    bytes: &[u8],
) -> Attempt {
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([bytes]).unwrap(),
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("append {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(_) => panic!("append {ordinal} denied"),
        TransitionOutcome::Deferred(_) => panic!("append {ordinal} deferred"),
        TransitionOutcome::Stale(_) => panic!("append {ordinal} stale"),
        TransitionOutcome::RebindRequired(_) => panic!("append {ordinal} rebind"),
        TransitionOutcome::Failed(_) => panic!("append {ordinal} failed"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(done) => {
            Attempt::Published(done.into_acknowledgment().record_ids().next().unwrap())
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => Attempt::NoEffect(fate.cause()),
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "append {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}
