pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator,
    worth_store::physical_runtime::PhysicalLocatorReadmissionDenial,
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordBootstrapDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordServingTerminalPosture,
};
pub(crate) use {
    worth_store_physical_format::PhysicalFreeSpaceMembershipBlock,
    worth_store_physical_format::RecordAllocationClass,
    worth_store_physical_format::RecordFreeSpaceManifestEntry,
};

pub(crate) use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) fn assert_mutation_fenced(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
) {
    assert!(matches!(
        worth_store_testkit_store::durable_publication::wal_data::prepare_single(
            &serving.certification_record_submission(),
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([209; 32]),
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
                b"must stay sealed".as_slice()
            ])
            .unwrap(),
        )
        .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::ServingRequiresInspection
            )
        )
    ));
}

pub(crate) fn assert_current_root_damage_denies_bootstrap(
    root: &std::path::Path,
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    access: worth_store::physical_runtime::AdmittedRecordAccessPolicy,
) {
    let outcome = open_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    })
    .into_raw();
    let worth_proof::TransitionOutcome::Denied(denial) = outcome else {
        panic!("an unreferenced free-space or routing block must deny Store bootstrap");
    };
    assert_eq!(
        denial.reason(),
        worth_store::physical_runtime::RecordBootstrapDenial::CurrentRootDamaged
    );
    denial.into_runtime().close();
}

pub(crate) fn overwrite_free_space_root_block(
    root: &std::path::Path,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    header: &worth_store_physical_format::DurableFreeSpaceManifestHeader,
    entries: Vec<RecordFreeSpaceManifestEntry>,
) {
    let reference = header.root().expect("the specimen has a free-space root");
    assert_eq!(
        reference.level(),
        0,
        "the focused specimen must have one leaf"
    );
    let block = worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::leaf(
        header.tree_identity(),
        reference.generation(),
        reference.block(),
        entries,
        header.node_capacity(),
    )
    .unwrap();
    std::fs::write(
        root.join(format!(
            "families/records/free-space/free-space-{:016x}-block-{:016x}.manifest",
            reference.generation(),
            reference.block(),
        )),
        block.encode(format),
    )
    .unwrap();
}
