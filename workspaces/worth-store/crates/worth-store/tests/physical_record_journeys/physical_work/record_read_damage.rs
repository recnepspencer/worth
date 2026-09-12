use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalManifestCapacityTransition, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, PhysicalWorkEffectFate, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition, RecordAppendBatch, RecordAppendDenial, RecordByteLimit,
    RecordReadDenial, RecordReadLimits,
};
use worth_store_physical_backend::{MediaFaultDirective, MediaOperationRole};

use super::super::{durable_publication, serving_from_open};
use super::record_read_signal_cleanup::await_read_signal_cleanup;
use super::{configuration, serving_from_initialization};

const PAYLOAD: &[u8] = b"canonical partial record read";

#[test]
fn partial_backend_read_is_denied_at_the_public_read_boundary_and_revokes_health() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("record-read-partial-damage", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let calibration = serving_from_open(&root);
    let bootstrap_reads = calibration
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedRead);
    calibration.close();

    let serving = super::fault_fixture::serving_from_open_with_positioned_read_fault(
        &root,
        bootstrap_reads + 1,
        MediaFaultDirective::AllowPrefix { bytes: 3 },
    );
    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
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
    assert_eq!(failure.denial(), RecordReadDenial::ArtifactDamaged);
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
    assert_eq!(partition.partition.0, "store.physical.record.root");
    assert_eq!(failed.effect_fate(), PhysicalWorkEffectFate::ReadIncomplete);
    assert_eq!(
        failed.recovery(),
        PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert!(
        failed.backend_operation().is_some(),
        "a real partial read retains its backend effect identity"
    );
    assert_eq!(
        after.attempts_for(MediaOperationRole::PositionedRead)
            - before.attempts_for(MediaOperationRole::PositionedRead),
        1
    );
    assert_eq!(
        after.completed_bytes_for(MediaOperationRole::PositionedRead)
            - before.completed_bytes_for(MediaOperationRole::PositionedRead),
        3
    );
    assert!(matches!(
        serving.records().expect("read protection admission").open(record, limits),
        Err(error) if error.denial() == RecordReadDenial::ServingRequiresInspection
    ));
    assert!(serving.close_plan().execute().requires_inspection());
}

#[test]
fn truncated_segment_is_structural_damage_before_range_dispatch_and_revokes_health() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("record-read-indeterminate-damage", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
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
    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
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
    assert_eq!(failure.denial(), RecordReadDenial::ArtifactDamaged);
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
    assert_eq!(failed.effect_fate(), PhysicalWorkEffectFate::ReadCompleted);
    assert_eq!(
        failed.operation(),
        PhysicalWorkOperationFamily::ArtifactMetadataRead,
        "the rejected structural observation must be the final segment operation"
    );
    assert_eq!(failed.recovery(), PhysicalWorkRecoveryDisposition::NoEffect);
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
            .attempts_for(MediaOperationRole::PositionedRead)
            - media_before.attempts_for(MediaOperationRole::PositionedRead),
        2,
        "only the two prerequisite locator reads may dispatch; the known-short segment must not"
    );
    assert!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::ReadMetadata)
            > media_before.attempts_for(MediaOperationRole::ReadMetadata)
    );
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.record_submission(),
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([214; 32]),
            RecordAppendBatch::try_from_iter([b"fenced".as_slice()]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert!(serving.close_plan().execute().requires_inspection());
}
