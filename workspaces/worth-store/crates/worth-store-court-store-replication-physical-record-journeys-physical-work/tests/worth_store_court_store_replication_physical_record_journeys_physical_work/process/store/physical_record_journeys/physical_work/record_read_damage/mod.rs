#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::record_read_damage::*;

use worth_store_testkit_store::physical_record_journeys::serving_from_open;
use worth_store_testkit_store::physical_work::record_read_signal_cleanup::await_read_signal_cleanup;
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn partial_backend_read_is_denied_at_the_public_read_boundary_and_revokes_health() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "record-read-partial-damage",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let calibration = serving_from_open(&root);
    let bootstrap_reads = calibration
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead);
    calibration.close();

    let serving = worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_read_fault(
        &root,
        bootstrap_reads + 1,
        worth_store_physical_backend::MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap(),
    );
    let before = serving.media_counters();
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let failure = match serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
    {
        Ok(_) => panic!("a partial backend read must not construct a record session"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.denial(),
        worth_store::physical_runtime::RecordReadDenial::ArtifactDamaged
    );
    assert!(failure.observation().physical_work_count() > 0);
    await_read_signal_cleanup(&serving);
    let after = serving.media_counters();
    let invalidations_after = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    assert_eq!(
        invalidations_after,
        invalidations_before + 1,
        "the admitted failing read must emit exactly one dependency invalidation"
    );
    let failed_identity = failure
        .observation()
        .last_physical_work()
        .expect("the failing physical read retains its identity");
    let causal = serving.physical_work_observer().causal().records();
    let failed = causal
        .iter()
        .find(|record| record.identity() == failed_identity)
        .expect("the failing physical read has causal settlement evidence");
    let bindings = serving.physical_signal_aspect_binding_observations();
    let partition = bindings
        .iter()
        .find(|binding| binding.digest() == failed.signal_binding())
        .and_then(|binding| binding.partition())
        .expect("the failing read binding is partitioned");
    assert_eq!(
        partition.path().segments()[0],
        "store.physical.record.frame"
    );
    assert_eq!(
        failed.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ReadIncomplete
    );
    assert_eq!(
        failed.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert!(
        failed.backend_operation().is_some(),
        "a real partial read retains its backend effect identity"
    );
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        1
    );
    assert_eq!(
        after.completed_bytes_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - before.completed_bytes_for(
                worth_store_physical_backend::MediaOperationRole::PositionedRead
            ),
        3
    );
    assert!(matches!(
        serving.records().expect("read protection admission").open(record, limits),
        Err(error) if error.denial() == worth_store::physical_runtime::RecordReadDenial::ServingRequiresInspection
    ));
    assert!(serving.close_plan().execute().requires_inspection());
}

#[test]
fn truncated_segment_is_structural_damage_before_range_dispatch_and_revokes_health() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "record-read-indeterminate-damage",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();
    std::fs::OpenOptions::new()
        .write(true)
        .open(
            root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages"),
        )
        .unwrap()
        .set_len(0)
        .unwrap();

    let serving = serving_from_open(&root);
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap(),
    );
    let media_before = serving.media_counters();
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let failure = match serving
        .records()
        .expect("read protection admission")
        .open(record, limits)
    {
        Ok(_) => panic!("a truncated admitted segment cannot produce a read session"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.denial(),
        worth_store::physical_runtime::RecordReadDenial::ArtifactDamaged
    );
    await_read_signal_cleanup(&serving);

    let failed_identity = failure
        .observation()
        .last_physical_work()
        .expect("structural damage retains its exact metadata work identity");
    let causal = serving.physical_work_observer().causal().records();
    let failed = causal
        .iter()
        .find(|record| record.identity() == failed_identity)
        .expect("the length observation has causal settlement evidence");
    assert_eq!(
        failed.effect_fate(),
        worth_store::physical_runtime::PhysicalWorkEffectFate::ReadCompleted
    );
    assert_eq!(
        failed.operation(),
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactMetadataRead,
        "the rejected structural observation must be the final segment operation"
    );
    assert_eq!(
        failed.recovery(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::NoEffect
    );
    assert!(
        failed.backend_operation().is_some(),
        "the contradictory file length comes from a real metadata effect"
    );
    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .aspect_invalidation_count(),
        invalidations_before + 1
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
            - media_before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        0,
        "captured-root routing needs no locator reads; the known-short segment must not dispatch a range read"
    );
    assert!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::ReadMetadata)
            > media_before
                .attempts_for(worth_store_physical_backend::MediaOperationRole::ReadMetadata)
    );
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([214; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"fenced".as_slice()])
                .unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
            )
        )
    ));
    assert!(serving.close_plan().execute().requires_inspection());
}
