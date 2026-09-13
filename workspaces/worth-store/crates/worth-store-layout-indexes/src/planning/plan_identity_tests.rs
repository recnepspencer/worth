use worth_store_budgets::{PreExecutionBudgetEnvelope, PreExecutionBudgetScope};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalReferenceAuthority,
    PhysicalRootReference, PhysicalSegmentId,
};
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

use super::AccessPlanSelector;
use crate::strategy::tests_support::admit_strategy_scope;

#[test]
fn selected_read_authority_is_a_compact_handle_to_native_plan_identity() {
    use std::mem::size_of;

    assert_eq!(size_of::<super::AccessPlanIdentity>(), size_of::<usize>());
    for (name, size) in [
        ("B-tree lookup", size_of::<super::SelectedBTreeLookup>()),
        ("LSM lookup", size_of::<super::SelectedLsmLookup>()),
        (
            "degraded exact scan",
            size_of::<super::SelectedDegradedExactScan>(),
        ),
    ] {
        assert!(
            size <= 256,
            "{name} selected authority embedded {size} bytes instead of retaining a compact identity handle"
        );
    }
}

#[test]
fn plan_identity_equality_includes_exact_admitted_budget_posture() {
    let (family, domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let materialization = crate::access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .expect("catalog root must admit exact materialization");
    let select = |budget| {
        let key = crate::keyspace::admit_page_key(
            domain,
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .expect("page key must admit");
        let request = AccessPlanSelector
            .admit_read_request(
                family,
                key,
                materialization.clone(),
                crate::access_planning().point_access(),
            )
            .expect("point request must admit");
        AccessPlanSelector
            .select_admitted_with_budget(request, budget)
            .into_btree_lookup()
            .expect("point request must select B-tree lookup")
    };

    let broad = select(PreExecutionBudgetEnvelope::foreground_default());
    let replayed = select(PreExecutionBudgetEnvelope::foreground_default());
    let exact = select(PreExecutionBudgetEnvelope::new(
        PreExecutionBudgetScope::Foreground,
        8_192,
        2,
        0,
        0,
        8_192,
    ));

    assert_eq!(broad.fingerprint(), replayed.fingerprint());
    assert_ne!(broad.fingerprint(), exact.fingerprint());
    assert_eq!(
        exact.fingerprint().budget_request(),
        exact.budget_receipt().request()
    );
    assert_eq!(
        exact.fingerprint().budget_envelope(),
        exact.budget_receipt().admitted_envelope()
    );
    assert_eq!(exact.fingerprint().cost_estimate(), exact.cost_estimate());
}

#[test]
fn equal_shaped_plans_from_distinct_physical_sources_have_distinct_identity() {
    let (family, domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let select = |generation| {
        let root = PhysicalGenerationAuthority::for_canonical_physical_format()
            .root_publication_cell(PhysicalRootReference::from_raw(1).unwrap())
            .with_root_publication_generation(PhysicalGeneration::from_raw(generation).unwrap());
        let admitted = PhysicalReferenceAuthority::for_canonical_physical_format()
            .admit_root_publication(root);
        let validated = PhysicalReferenceAuthority::for_canonical_physical_format()
            .validate_root_publication(admitted, root)
            .expect("root publication must validate");
        let materialization = crate::access_planning()
            .admit_btree_publication_materialization(family, &catalog, validated)
            .expect("root source must admit materialization");
        let key = crate::keyspace::admit_page_key(
            domain,
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .expect("page key must admit");
        let request = AccessPlanSelector
            .admit_read_request(
                family,
                key,
                materialization,
                crate::access_planning().point_access(),
            )
            .expect("point request must admit");
        AccessPlanSelector
            .select_admitted_with_budget(request, PreExecutionBudgetEnvelope::foreground_default())
            .into_btree_lookup()
            .expect("point request must select B-tree lookup")
    };

    let first = select(1);
    let replayed = select(1);
    let different_source = select(2);

    assert_eq!(first.fingerprint(), replayed.fingerprint());
    assert_ne!(first.materialization(), different_source.materialization());
    assert_ne!(first.fingerprint(), different_source.fingerprint());
}
