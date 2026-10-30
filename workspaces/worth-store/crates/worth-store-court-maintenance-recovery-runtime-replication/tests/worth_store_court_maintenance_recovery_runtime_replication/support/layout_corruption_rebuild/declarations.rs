pub(crate) use worth_store_contracts::DurableArtifactFamilyId;
pub(crate) use worth_store_security::admitted_tenant_page_security_scope_for_layout_partition_test;
pub(crate) use {
    worth_store_layout_indexes::access_planning, worth_store_layout_indexes::access_shapes,
    worth_store_layout_indexes::layout_parity_verification,
    worth_store_layout_indexes::layout_rebuild_admission,
    worth_store_layout_indexes::layout_rebuild_candidate_readmission,
    worth_store_layout_indexes::layout_rebuild_execution,
    worth_store_layout_indexes::AdmittedLayoutMaterialization,
    worth_store_layout_indexes::DerivedIndexCandidateDeclaration,
    worth_store_layout_indexes::DerivedIndexParityBasis,
    worth_store_layout_indexes::DerivedIndexParityRow,
    worth_store_layout_indexes::DerivedIndexRebuildRequest,
    worth_store_layout_indexes::DerivedIndexRebuildSourceInput,
    worth_store_layout_indexes::IndexMaintenanceMode,
    worth_store_layout_indexes::LayoutStrategyRegistrySnapshot,
    worth_store_layout_indexes::PhysicalMutationShape,
};
pub(crate) use {
    worth_store_layout_indexes::declarations::layout_declarations,
    worth_store_layout_indexes::declarations::ArtifactFamilyAccessLane,
};
pub(crate) use {
    worth_store_layout_indexes::strategy_declarations::layout_admission_registry,
    worth_store_layout_indexes::strategy_declarations::LayoutAdmissionRequest,
    worth_store_layout_indexes::strategy_declarations::LayoutRequestedCapability,
    worth_store_layout_indexes::strategy_declarations::LayoutStrategyFamily,
};
pub(crate) use {
    worth_store_physical_format::PhysicalReferenceAuthority,
    worth_store_physical_format::PhysicalRootManifestRebuildSource,
    worth_store_physical_format::PhysicalStoreIdentity,
};

pub(crate) fn execute_root_rebuild(
    page: u64,
) -> worth_store_layout_indexes::DerivedIndexRebuildReceipt {
    let strategy = btree_strategy();
    let source = root_source(page);
    let materialization = root_materialization(&strategy, &source);
    let request = worth_store_layout_indexes::DerivedIndexRebuildRequest::new(
        strategy.admitted_strategy().admitted_family(),
        strategy.admitted_strategy().admitted_key_domain(),
        strategy.admitted_strategy().family(),
        access_shapes()
            .rebuild_read_declaration(
                worth_store_layout_indexes::AccessLaneClassification::Maintenance,
            )
            .expect("maintenance declares rebuild reads"),
        materialization,
        worth_store_layout_indexes::DerivedIndexRebuildSourceInput::PhysicalRootManifest { source },
    );
    let plan = layout_rebuild_admission()
        .admit_plan(request)
        .into_admitted()
        .expect("ordinary root rebuild must admit");
    layout_rebuild_execution().execute(plan).into_rebuilt()
}

pub(crate) fn btree_strategy() -> LayoutStrategyRegistrySnapshot {
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let declaration = layout_declarations()
        .declaration(worth_store_contracts::DurableArtifactFamilyId::PhysicalPage)
        .expect("physical pages declare a layout");
    let family = layout_declarations()
        .admit_physical_artifact_family(declaration, security.witnesses())
        .into_result()
        .expect("current page scope admits its family");
    let domain = layout_declarations()
        .admit_physical_key_domain(family, security.witnesses())
        .into_result()
        .expect("current page scope admits its key domain");
    layout_admission_registry()
        .admit(
            worth_store_layout_indexes::strategy_declarations::LayoutAdmissionRequest::from_admitted(
                family,
                domain,
                worth_store_layout_indexes::strategy_declarations::LayoutStrategyFamily::BTreeRange,
                worth_store_layout_indexes::strategy_declarations::LayoutRequestedCapability::point_lookup(),
                worth_store_layout_indexes::declarations::ArtifactFamilyAccessLane::HotPath,
            )
            .for_mutation_shape(worth_store_layout_indexes::PhysicalMutationShape::PointRewrite)
            .under_maintenance_mode(worth_store_layout_indexes::IndexMaintenanceMode::SynchronousExact),
        )
        .into_result()
        .expect("ordinary B-tree strategy must admit")
}

pub(crate) fn root_source(page: u64) -> PhysicalRootManifestRebuildSource {
    worth_store_test_support::execute_root_manifest_rebuild_source(
        &worth_store_physical_format::PhysicalStoreIdentity::physical_format_default(),
        7,
        page,
        1,
    )
}

pub(crate) fn root_materialization(
    strategy: &LayoutStrategyRegistrySnapshot,
    source: &PhysicalRootManifestRebuildSource,
) -> AdmittedLayoutMaterialization {
    let catalog = worth_store_test_support::admitted_layout_bootstrap_catalog();
    let publication = source.witness().manifest().root_publication();
    let references =
        worth_store_physical_format::PhysicalReferenceAuthority::for_canonical_physical_format();
    let validated = references
        .validate_root_publication(references.admit_root_publication(publication), publication)
        .expect("Store-issued source contains a valid root publication");
    access_planning()
        .admit_btree_publication_materialization(
            strategy.admitted_strategy().admitted_family(),
            &catalog,
            validated,
        )
        .into_result()
        .expect("published root admits exact rebuild materialization")
}
