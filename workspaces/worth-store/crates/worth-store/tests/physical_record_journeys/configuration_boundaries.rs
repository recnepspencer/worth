use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalManifestCapacityTransition,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationPreparationDenial, PhysicalPageSizeClass,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial,
    RecordBootstrapDenial, RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::DurableExtentManifest;

use super::{
    configuration, durable_publication, media, read_record,
    scenario_configuration::dense_configuration, serving_from_initialization, success,
};

fn format_64k() -> AdmittedPhysicalRecordFormat {
    AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder()
            .page_size(PhysicalPageSizeClass::KiB64)
            .admit()
            .unwrap(),
    )
}

#[test]
fn cross_format_configuration_is_denied_before_initialization_or_open_effects() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format_16k, placement_16k, access_16k) = configuration();
    let access_64k = PhysicalRecordAccessPolicy::builder()
        .admit(format_64k())
        .unwrap();
    let initialize = initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format_16k, placement_16k, access_64k, durability)
    })
    .into_raw();
    let TransitionOutcome::Denied(denial) = initialize else {
        panic!("cross-format initialization must be denied");
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::ConfigurationMismatch
    );
    assert!(!root.join("families/records").exists());
    let runtime = denial.into_runtime();

    let serving = match initialize_record_store!(runtime, |durability| {
        PhysicalRecordInitialization::new(format_16k, placement_16k, access_16k, durability)
    })
    .into_raw()
    {
        TransitionOutcome::Success(serving) => serving,
        _ => panic!("matching configuration must initialize"),
    };
    serving.close();
    let open = open_record_store!(media(&root), |durability| PhysicalRecordOpen::new(
        format_16k, access_64k, durability
    ))
    .into_raw();
    let TransitionOutcome::Denied(denial) = open else {
        panic!("cross-format open must be denied");
    };
    assert_eq!(
        denial.reason(),
        RecordBootstrapDenial::ConfigurationMismatch
    );
    denial.into_runtime().close();
}

#[test]
fn cross_format_placement_cannot_publish_an_unreopenable_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let placement_64k = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(700).unwrap())
        .admit(format_64k())
        .unwrap();
    let serving = serving_from_initialization(&root);
    let before = serving.media_counters();
    assert!(matches!(
        durable_publication::prepare_single(
            &serving.record_submission(),
            placement_64k,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            PhysicalMutationIdempotencyMaterial::new([212; 32]),
            RecordAppendBatch::try_from_iter([b"wrong format".as_slice()]).unwrap(),
        )
        .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::RecordAppend(
            RecordAppendDenial::PlacementFormatMismatch
        ))
    ));
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(MediaOperationRole::PositionedWrite),
        before.attempts_for(MediaOperationRole::PositionedWrite)
    );
    assert_eq!(after.replacements(), before.replacements());
    serving.close();
}

#[test]
fn extent_geometry_is_format_owned_and_survives_access_policy_narrowing() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, _) = dense_configuration(4);
    let wide_access = PhysicalRecordAccessPolicy::builder()
        .transfer_limit(RecordByteLimit::new(65_536).unwrap())
        .scratch_limit(RecordByteLimit::new(65_536).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, wide_access, durability)
    }));
    let payload = vec![0x5a; 40_000];
    let published = durable_publication::publish_single(
        &serving,
        placement,
        durable_publication::certification_material("extent-geometry-access-narrowing", 1),
        RecordAppendBatch::try_from_iter([payload.as_slice()]).unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let manifest_bytes = std::fs::read(root.join(
        "families/records/extent-manifests/extent-0000000000000001-0000000000000001.manifest",
    ))
    .unwrap();
    let (manifest, _) = DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.maximum_frame_bytes(), 16_384);

    let (_, _, narrow_access) = configuration();
    let reopened = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, narrow_access, durability)
    }));
    let session = reopened
        .records()
        .expect("read protection admission")
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(40_000).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(session, payload.len()).0, payload);
    reopened.close();
}
