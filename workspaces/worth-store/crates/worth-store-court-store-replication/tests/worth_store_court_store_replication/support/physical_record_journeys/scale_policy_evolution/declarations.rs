pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::ExternalPhysicalRecordLocator, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordAppendDenial};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};

pub(crate) use {worth_store_testkit_store::durable_publication::wal_data::prepare_single, worth_store_testkit_store::durable_publication::root::publish_single, worth_store_testkit_store::durable_publication::root::publish_single_with_manifest_capacity_transition};

pub(crate) use {worth_store_testkit_store::scale_support::access, worth_store_testkit_store::scale_support::assert_canonical_parity, worth_store_testkit_store::scale_support::complete_scan, worth_store_testkit_store::scale_support::format, worth_store_testkit_store::scale_support::placement};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn prove() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("policy");
    let format = format();
    let initial = placement(format, 4, 2, 50);
    let admitted_access = access(format, 11);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, initial, admitted_access, durability)
    }));
    let first = publish_single(
        &serving,
        initial,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([171; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter((0..25).map(|value| vec![value; 600])).unwrap(),
    );
    let stable = first.settled_members()[0].record_id(0).unwrap();
    let locator = worth_store::physical_runtime::ExternalPhysicalRecordLocator::new(serving.store_identity(), stable);

    let wider = placement(format, 8, 3, 75);
    let writes_before = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    assert!(matches!(
        prepare_single(
            &serving.record_submission(),
            wider,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([172; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"cost-visible migration".as_slice()]).unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
            worth_store::physical_runtime::RecordAppendDenial::ManifestCapacityMigrationRequired
        ))
    ));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes_before
    );
    publish_single_with_manifest_capacity_transition(
        &serving,
        wider,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::ReconstructToRequested,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([173; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter((0..5).map(|value| vec![0x80 + value; 900])).unwrap(),
    );
    let wide = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    assert_eq!(wide.node_capacity(), 8);

    let narrower = placement(format, 2, 1, 60);
    publish_single_with_manifest_capacity_transition(
        &serving,
        narrower,
        worth_store::physical_runtime::PhysicalManifestCapacityTransition::ReconstructToRequested,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([174; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter((0..5).map(|value| vec![0xc0 + value; 700])).unwrap(),
    );
    assert_eq!(
        serving
            .records()
            .expect("read protection admission")
            .readmit_locator(locator)
            .into_result()
            .unwrap(),
        stable
    );
    let narrow = worth_store_offline_verifier::walk_current_durable_record_manifest(
        &root,
        format.declaration(),
    )
    .unwrap();
    assert_eq!(narrow.node_capacity(), 2);
    assert_eq!(narrow.placements().len(), 35);
    assert_canonical_parity(&serving, &narrow);
    let scan = complete_scan(&serving, 11, 131_072);
    assert_eq!(scan.records(), 35);
    serving.close();

    let changed_access = access(format, 3);
    let reopened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, changed_access, durability)
    }));
    assert_eq!(
        reopened
            .records()
            .expect("read protection admission")
            .readmit_locator(locator)
            .into_result()
            .unwrap(),
        stable
    );
    reopened.close();
}