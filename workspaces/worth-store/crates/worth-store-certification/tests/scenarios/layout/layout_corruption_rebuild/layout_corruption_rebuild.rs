use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_layout_indexes::declarations::{layout_declarations, ArtifactFamilyAccessLane};
use worth_store_layout_indexes::strategy_declarations::{
    layout_admission_registry, LayoutAdmissionRequest, LayoutRequestedCapability,
    LayoutStrategyFamily,
};
use worth_store_layout_indexes::{
    access_planning, access_shapes, layout_parity_verification, layout_rebuild_admission,
    layout_rebuild_candidate_readmission, layout_rebuild_execution, AdmittedLayoutMaterialization,
    DerivedIndexCandidateDeclaration, DerivedIndexParityBasis, DerivedIndexParityRow,
    DerivedIndexRebuildRequest, DerivedIndexRebuildSourceInput, IndexMaintenanceMode,
    LayoutStrategyRegistrySnapshot, PhysicalMutationShape,
};
use worth_store_physical_format::{
    PhysicalReferenceAuthority, PhysicalRootManifestRebuildSource, PhysicalStoreIdentity,
};
use worth_store_security::admitted_tenant_page_security_scope_for_layout_partition_test;

#[test]
fn root_manifest_rebuild_reaches_exact_parity_through_production_owners() {
    let execution = execute_root_rebuild(11);
    let candidate = execution.candidate_declaration();
    let readmitted = layout_rebuild_candidate_readmission().readmit(execution, candidate);
    let parity = layout_parity_verification().verify(readmitted);

    assert_eq!(parity.case_id().as_str(), "verified");
}

#[test]
fn copied_rebuild_shape_with_changed_value_is_denied_at_parity() {
    let execution = execute_root_rebuild(11);
    let exact = execution.rebuilt_basis();
    let hostile = DerivedIndexParityBasis::new(
        vec![DerivedIndexParityRow::new(
            exact.ordered_rows()[0].key().clone(),
            "sha256:hostile-rebuild-value",
        )],
        exact.coverage().clone(),
        exact.cost_envelope_compliant(),
        exact.counter_shape().to_vec(),
    )
    .expect("hostile candidate remains canonically shaped");
    let readmitted = layout_rebuild_candidate_readmission().readmit(
        execution,
        DerivedIndexCandidateDeclaration::from_canonical_basis(hostile),
    );
    let parity = layout_parity_verification().verify(readmitted);

    assert_eq!(parity.case_id().as_str(), "denied.value_identity");
}

fn execute_root_rebuild(page: u64) -> worth_store_layout_indexes::DerivedIndexRebuildReceipt {
    let strategy = btree_strategy();
    let source = root_source(page);
    let materialization = root_materialization(&strategy, &source);
    let request = DerivedIndexRebuildRequest::new(
        strategy.admitted_strategy().admitted_family(),
        strategy.admitted_strategy().admitted_key_domain(),
        strategy.admitted_strategy().family(),
        access_shapes()
            .rebuild_read_declaration(
                worth_store_layout_indexes::AccessLaneClassification::Maintenance,
            )
            .expect("maintenance declares rebuild reads"),
        materialization,
        DerivedIndexRebuildSourceInput::PhysicalRootManifest { source },
    );
    let plan = layout_rebuild_admission()
        .admit_plan(request)
        .into_admitted()
        .expect("ordinary root rebuild must admit");
    layout_rebuild_execution().execute(plan).into_rebuilt()
}

fn btree_strategy() -> LayoutStrategyRegistrySnapshot {
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let declaration = layout_declarations()
        .declaration(DurableArtifactFamilyId::PhysicalPage)
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
            LayoutAdmissionRequest::from_admitted(
                family,
                domain,
                LayoutStrategyFamily::BaselineBTreeRange,
                LayoutRequestedCapability::point_lookup(),
                ArtifactFamilyAccessLane::HotPath,
            )
            .for_mutation_shape(PhysicalMutationShape::PointRewrite)
            .under_maintenance_mode(IndexMaintenanceMode::SynchronousExact),
        )
        .into_result()
        .expect("ordinary B-tree strategy must admit")
}

fn root_source(page: u64) -> PhysicalRootManifestRebuildSource {
    worth_store_test_support::execute_root_manifest_rebuild_source(
        &PhysicalStoreIdentity::physical_format_default(),
        7,
        page,
        1,
    )
}

fn root_materialization(
    strategy: &LayoutStrategyRegistrySnapshot,
    source: &PhysicalRootManifestRebuildSource,
) -> AdmittedLayoutMaterialization {
    let catalog = worth_store_test_support::admitted_layout_bootstrap_catalog();
    let publication = source.witness().manifest().root_publication();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
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
