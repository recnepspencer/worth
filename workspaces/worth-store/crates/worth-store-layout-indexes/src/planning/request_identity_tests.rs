use crate::facade::{access_planning, deterministic_plan_selection};
use crate::strategy::tests_support::{
    admit_persisted_lsm_scope, admit_strategy_scope, persisted_lsm_materialization,
};
use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_contracts::WalRecordFamily;
use worth_store_physical_format::PhysicalEpoch;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};
use worth_store_wal::StoreWalRecordIdentity;

#[test]
fn concrete_request_identity_changes_plan_binding_for_equal_shaped_wal_reads() {
    let (lifecycle, key_domain) = admit_persisted_lsm_scope();
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let shape = access_planning().point_access();
    let materialization = persisted_lsm_materialization(lifecycle, &catalog).0;
    let wal_key = |id| {
        crate::keyspace::admit_wal_key(
            key_domain,
            WalRecordFamily::DurableMutationIntent,
            StoreWalRecordIdentity::new(id),
        )
        .unwrap()
    };
    let first_key = wal_key(7);
    let second_key = wal_key(8);

    let first_request = deterministic_plan_selection()
        .admit_read_request(lifecycle, first_key, materialization.clone(), shape)
        .unwrap();
    let first = deterministic_plan_selection()
        .select_admitted_with_budget(
            first_request,
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .into_lsm_lookup()
        .expect("WAL point request must select LSM lookup");
    let second_request = deterministic_plan_selection()
        .admit_read_request(lifecycle, second_key, materialization.clone(), shape)
        .unwrap();
    let second = deterministic_plan_selection()
        .select_admitted_with_budget(
            second_request,
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .into_lsm_lookup()
        .expect("WAL point request must select LSM lookup");

    assert_ne!(first.request_identity(), second.request_identity());
    assert_ne!(first.fingerprint(), second.fingerprint());
    assert_eq!(first.materialization(), &materialization);
    assert_eq!(
        first.materialization().coverage().source(),
        materialization.source(),
        "ordinary test selection retains the admitted physical source",
    );
    assert_eq!(
        first.fingerprint().materialization(),
        Some(&materialization)
    );
    assert_eq!(
        first.cost_estimate().materialization_source(),
        Some(materialization.source()),
    );
    assert_eq!(
        first.cost_estimate().exact_coverage(),
        Some(materialization.coverage()),
    );
    let admitted_strategy = first.admitted_strategy();
    assert_eq!(admitted_strategy.admitted_family(), lifecycle);
    assert_eq!(admitted_strategy.admitted_key_domain(), key_domain);
    assert_eq!(
        first.fingerprint().admitted_strategy(),
        Some(admitted_strategy)
    );
    let strategy_admission = first.strategy_admission();
    assert_eq!(
        strategy_admission.request().exact_coverage(),
        Some(materialization.coverage())
    );
    assert_eq!(
        first.fingerprint().strategy_admission(),
        Some(strategy_admission),
    );
}

#[test]
fn ordinary_catalog_materialization_retains_its_exact_source_identity() {
    let (family, _) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let materialization = access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .unwrap();

    assert_eq!(
        materialization.coverage().source(),
        materialization.source()
    );
    assert_eq!(materialization.source_root_owner(), catalog.root_owner());
    assert_eq!(
        materialization.source_format_version(),
        catalog.physical_format_version(),
    );
}

#[test]
fn advanced_catalog_frontier_rejects_old_exact_materialization() {
    let (family, _) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let original = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let advanced = crate::bootstrap::test_support::advanced_bootstrap_catalog_read_admission();
    assert_ne!(original.root_owner(), advanced.root_owner());
    let materialization = access_planning()
        .admit_current_catalog_root_materialization(family, &original)
        .unwrap();
    let advanced_frontier = access_planning().current_materialization_frontier(&advanced);
    let stale = match materialization
        .clone()
        .classify_freshness_at(advanced_frontier.clone())
        .unwrap()
    {
        crate::MaterializationFreshness::Stale(stale) => stale,
        crate::MaterializationFreshness::Current(_) => panic!("advanced root must be stale"),
    };
    assert_eq!(stale.materialization(), &materialization);
    assert_eq!(stale.observed_frontier(), &advanced_frontier);

    assert_eq!(
        materialization
            .clone()
            .require_current_at(advanced_frontier),
        Err(crate::MaterializationDenial::MaterializationFrontierMismatch),
    );
}

#[test]
fn shape_declaration_cannot_substitute_admitted_materialization_coverage() {
    let (family, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let admitted_coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                family.declaration().family(),
            ),
            PhysicalEpoch::from_raw(31).unwrap(),
        )
        .unwrap();
    let substituted_coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                family.declaration().family(),
            ),
            PhysicalEpoch::from_raw(32).unwrap(),
        )
        .unwrap();
    let materialization = admitted_materialization(family, admitted_coverage);
    let substituted_shape = access_planning().point_access();

    let admitted = deterministic_plan_selection()
        .admit_read_request(
            family,
            page_key(key_domain, 7),
            materialization.clone(),
            substituted_shape,
        )
        .expect("shape declaration does not author materialization coverage");
    assert_eq!(admitted.materialization(), &materialization);
    assert_ne!(admitted.materialization().coverage(), &substituted_coverage,);
}

#[test]
fn request_admission_rejects_key_domain_from_another_artifact_family() {
    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let (_, segment_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalSegment,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                page_lifecycle.declaration().family(),
            ),
            PhysicalEpoch::from_raw(31).unwrap(),
        )
        .unwrap();
    let shape = access_planning().point_access();
    let materialization = admitted_materialization(page_lifecycle, coverage);
    let segment_key = crate::keyspace::admit_segment_key(
        segment_domain,
        crate::keyspace::tests_support::segment_id(7),
    )
    .unwrap();

    assert_eq!(
        deterministic_plan_selection().admit_read_request(
            page_lifecycle,
            segment_key,
            materialization,
            shape
        ),
        Err(crate::PhysicalAccessRequestAdmissionDenied::KeyDomainFamilyMismatch),
    );
    let _ = page_domain;
}

#[test]
fn request_admission_rejects_same_family_from_another_security_authority() {
    let (page_family, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let foreign_security =
        worth_store_security::admitted_store_managed_root_security_scope_for_layout_partition_test(
        );
    let foreign_family = crate::layout_declarations()
        .admit_physical_artifact_family(page_family.declaration(), foreign_security.witnesses())
        .unwrap();
    let coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                page_family.declaration().family(),
            ),
            PhysicalEpoch::from_raw(31).unwrap(),
        )
        .unwrap();
    let shape = access_planning().point_access();
    let materialization = admitted_materialization(page_family, coverage);

    assert_eq!(
        deterministic_plan_selection().admit_read_request(
            foreign_family,
            page_key(page_domain, 7),
            materialization,
            shape
        ),
        Err(crate::PhysicalAccessRequestAdmissionDenied::KeyDomainAuthorityMismatch),
    );
}

#[test]
fn request_admission_rejects_materialization_from_another_artifact_family() {
    let (page_lifecycle, page_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let (segment_lifecycle, _) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalSegment,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let foreign_coverage = access_planning()
        .exact_root_epoch_coverage(
            crate::bootstrap::test_support::bootstrap_exact_materialization(
                segment_lifecycle.declaration().family(),
            ),
            PhysicalEpoch::from_raw(31).unwrap(),
        )
        .unwrap();
    let foreign_shape = access_planning().point_access();
    let foreign_materialization = admitted_materialization(segment_lifecycle, foreign_coverage);

    assert_eq!(
        deterministic_plan_selection().admit_read_request(
            page_lifecycle,
            page_key(page_domain, 7),
            foreign_materialization,
            foreign_shape
        ),
        Err(crate::PhysicalAccessRequestAdmissionDenied::MaterializationFamilyMismatch),
    );
}

#[test]
fn request_admission_distinguishes_family_service_lane_from_operation_lane() {
    let (hot_lifecycle, hot_domain) = admit_persisted_lsm_scope();
    let hot_key = crate::layout_declarations()
        .admit_wal_key(
            hot_domain,
            WalRecordFamily::DurableMutationIntent,
            StoreWalRecordIdentity::new(17),
        )
        .unwrap();
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let maintenance_shape = crate::access_shapes()
        .rebuild_read(crate::AccessLaneClassification::Maintenance)
        .unwrap();
    let materialization = persisted_lsm_materialization(hot_lifecycle, &catalog).0;
    assert!(deterministic_plan_selection()
        .admit_recovery_request(hot_lifecycle, hot_key, materialization, maintenance_shape)
        .is_ok());

    let declarations = crate::layout_declarations();
    let maintenance_declaration = declarations
        .declaration(DurableArtifactFamilyId::WalBulkCheckpointPublicationIntent)
        .unwrap();
    let maintenance_authority = declarations
        .require_production_authority(declarations.classify_family(maintenance_declaration))
        .unwrap();
    assert_eq!(
        declarations.require_strategy_lifecycle(maintenance_authority),
        Err(crate::ArtifactFamilyDenial::ReadmissionFamilyCannotEnterStrategyAdmission),
    );
}

fn admitted_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
    coverage: crate::LayoutCoverageWitness,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    assert_eq!(
        coverage.upper_bound().basis_kind(),
        crate::materialization::CoverageBasisKind::RootEpoch
    );
    access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .unwrap()
}

fn page_key(
    key_domain: crate::AdmittedPhysicalKeyDomain,
    page: u64,
) -> crate::AdmittedConcretePhysicalKey {
    crate::layout_declarations()
        .admit_page_key(
            key_domain,
            crate::keyspace::tests_support::segment_id(4),
            crate::keyspace::tests_support::page_id(page),
        )
        .unwrap()
}
