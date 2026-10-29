use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial};
use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

#[test]
fn semantic_rejection_consumes_the_admitted_residency_projection_failure() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let coordinate =
        worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 0, 8).unwrap();
    let frame = serving
        .certification_physical_residency()
        .pin_exact(coordinate)
        .unwrap();
    assert_eq!(frame.physical_work_count(), 1);

    frame.reject_projection_failure();

    assert_eq!(
        serving
            .physical_signal_observation()
            .unwrap()
            .aspect_invalidation_count(),
        invalidations_before + 1
    );
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([212; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"fenced".as_slice()]).unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
            worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
        ))
    ));
    assert!(serving.close_plan().execute().requires_inspection());
}
