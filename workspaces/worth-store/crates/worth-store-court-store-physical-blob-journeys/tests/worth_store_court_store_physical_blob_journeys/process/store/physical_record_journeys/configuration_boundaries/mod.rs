#[allow(unused_imports)]
use crate::physical_record_journeys::configuration_boundaries::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::DurableExtentManifest;
use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalPageSizeClass,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn cross_format_configuration_is_denied_before_initialization_or_open_effects() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format_16k, placement_16k, access_16k) = configuration();
    let access_64k = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format_64k())
        .unwrap();
    let initialize = initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format_16k,
            placement_16k,
            access_64k,
            durability,
        )
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = initialize else {
        panic!("cross-format initialization must be denied");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::ConfigurationMismatch
    );
    assert!(!root.join("families/records").exists());
    let runtime = denial.into_runtime();

    let serving = match initialize_record_store!(runtime, |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format_16k,
            placement_16k,
            access_16k,
            durability,
        )
    })
    .into_raw()
    {
        worth_proof::TransitionOutcome::Success(serving) => serving,
        _ => panic!("matching configuration must initialize"),
    };
    serving.close();
    let open = open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format_16k, access_64k, durability)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = open else {
        panic!("cross-format open must be denied");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::ConfigurationMismatch
    );
    denial.into_runtime().close();
}

#[test]
fn cross_format_placement_cannot_publish_an_unreopenable_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let placement_64k = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(700).unwrap())
        .admit(format_64k())
        .unwrap();
    let serving = serving_from_initialization(&root);
    let before = serving.media_counters();
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.record_submission(),
            placement_64k,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([212; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"wrong format".as_slice()
            ])
            .unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::PlacementFormatMismatch
            )
        )
    ));
    let after = serving.media_counters();
    assert_eq!(
        after.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        before.attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
    );
    assert_eq!(after.replacements(), before.replacements());
    serving.close();
}

#[test]
fn extent_geometry_is_format_owned_and_survives_access_policy_narrowing() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, _) = dense_configuration(4);
    let wide_access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .transfer_limit(worth_store::physical_runtime::RecordByteLimit::new(65_536).unwrap())
        .scratch_limit(worth_store::physical_runtime::RecordByteLimit::new(65_536).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format,
            placement,
            wide_access,
            durability,
        )
    }));
    let payload = vec![0x5a; 40_000];
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "extent-geometry-access-narrowing",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([payload.as_slice()])
            .unwrap(),
    );
    let record = published.settled_members()[0].record_id(0).unwrap();
    serving.close();

    let manifest_bytes =
        std::fs::read(root.join("families/records/arenas/arena-0000000000000001.data")).unwrap();
    let (manifest, _) =
        worth_store_physical_format::DurableExtentManifest::decode(&manifest_bytes[..104]).unwrap();
    assert_eq!(manifest.maximum_frame_bytes(), 16_384);

    let (_, _, narrow_access) = configuration();
    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, narrow_access, durability)
    }));
    let session = reopened
        .records()
        .expect("read protection admission")
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(40_000).unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(read_record(session, payload.len()).0, payload);
    reopened.close();
}
