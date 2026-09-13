use crate::facade::{access_planning, deterministic_plan_selection};
use crate::strategy::tests_support::admit_strategy_scope;
use crate::{access_shapes, AccessPlanSelectionDenied};
use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

fn root_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
    _epoch: u64,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .expect("physical catalog must admit exact root materialization")
}

#[test]
fn exact_multi_range_and_grouped_prefix_paths_fail_closed_without_btree_counter_lane() {
    use crate::strategy::tests_support::admit_btree_page_strategy;
    use crate::{GroupedPrefixBasis, MultiRangeBasis};

    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let btree = admit_btree_page_strategy();

    assert_eq!(
        btree.planned_counter_envelope_for(
            crate::access::shape::AccessShapeDetail::MultiRangeLookup(
                MultiRangeBasis::DeclaredDisjointRangeSet,
            )
        ),
        None
    );
    assert_eq!(
        btree.planned_counter_envelope_for(
            crate::access::shape::AccessShapeDetail::GroupedPrefixLookup(
                GroupedPrefixBasis::CanonicalGroupedPrefixes,
            )
        ),
        None
    );
    assert_eq!(
        deterministic_plan_selection()
            .select_admitted_with_budget(
                crate::planning::AccessPlanSelector
                    .admit_read_request(
                        lifecycle,
                        crate::keyspace::admit_page_key(
                            key_domain,
                            worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
                            worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap()
                        )
                        .expect("page identity must pass ordinary key admission"),
                        root_materialization(lifecycle, 19),
                        access_shapes().multi_range_lookup_declaration(
                            MultiRangeBasis::DeclaredDisjointRangeSet
                        )
                    )
                    .expect("test request must pass ordinary admission"),
                PreExecutionBudgetEnvelope::foreground_default(),
            )
            .unwrap_err(),
        AccessPlanSelectionDenied::NoEligibleAlternative
    );
    assert_eq!(
        deterministic_plan_selection()
            .select_admitted_with_budget(
                crate::planning::AccessPlanSelector
                    .admit_read_request(
                        lifecycle,
                        crate::keyspace::admit_page_key(
                            key_domain,
                            worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
                            worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap()
                        )
                        .expect("page identity must pass ordinary key admission"),
                        root_materialization(lifecycle, 19),
                        access_shapes().grouped_prefix_lookup_declaration(
                            GroupedPrefixBasis::CanonicalGroupedPrefixes,
                        )
                    )
                    .expect("test request must pass ordinary admission"),
                PreExecutionBudgetEnvelope::foreground_default(),
            )
            .unwrap_err(),
        AccessPlanSelectionDenied::NoEligibleAlternative
    );
}
