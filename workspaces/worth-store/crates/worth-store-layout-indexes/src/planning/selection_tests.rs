use super::{DeterministicSelectionRule, SelectionCandidateEligibility};
use crate::facade::{access_planning, deterministic_plan_selection};
use crate::strategy::tests_support::{
    admit_persisted_lsm_scope, admit_strategy_scope, persisted_lsm_materialization,
};
use crate::{
    access_shapes, AccessPlanSelectionDenied, DegradedExactScanRequest, LayoutStrategyFamily,
    SelectionCandidateOutcome,
};
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

fn wal_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
    _lsn: u64,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    persisted_lsm_materialization(family, &catalog).0
}

#[test]
fn btree_planning_denies_without_store_observed_cost_basis() {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let page_key = || {
        crate::keyspace::admit_page_key(
            key_domain,
            worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap(),
        )
        .expect("page identity")
    };
    for shape in [
        access_planning().point_access(),
        access_planning().range_access(),
        access_planning().prefix_access(),
    ] {
        let request = crate::planning::AccessPlanSelector
            .admit_read_request(
                lifecycle,
                page_key(),
                root_materialization(lifecycle, 7),
                shape,
            )
            .expect("read request");
        let result = deterministic_plan_selection()
            .select_admitted_with_budget(request, PreExecutionBudgetEnvelope::foreground_default());
        assert_eq!(
            result.unwrap_err(),
            AccessPlanSelectionDenied::NoEligibleAlternative
        );
    }
    let recovery = crate::planning::AccessPlanSelector
        .admit_recovery_request(
            lifecycle,
            page_key(),
            root_materialization(lifecycle, 7),
            access_planning()
                .rebuild_access(crate::AccessLaneClassification::Maintenance)
                .expect("rebuild shape"),
        )
        .expect("recovery request");
    let result = deterministic_plan_selection()
        .select_admitted_with_budget(recovery, PreExecutionBudgetEnvelope::maintenance_default());
    assert_eq!(
        result.unwrap_err(),
        AccessPlanSelectionDenied::NoEligibleAlternative
    );
}
#[test]
fn deterministic_selection_selects_lsm_for_exact_wal_point_paths() {
    let (lifecycle, key_domain) = admit_persisted_lsm_scope();
    let access_shape = access_planning().point_access();

    let selected = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_read_request(
                    lifecycle,
                    crate::keyspace::admit_wal_key(
                        key_domain,
                        worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                        worth_store_wal::StoreWalRecordIdentity::new(1),
                    )
                    .expect("WAL identity must pass ordinary key admission"),
                    wal_materialization(lifecycle, 17),
                    access_shape,
                )
                .expect("test request must pass ordinary admission"),
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .into_lsm_lookup()
        .expect("WAL point request must issue LSM lookup authority");

    assert_eq!(
        selected.selected_family(),
        LayoutStrategyFamily::BaselineLsmWriteOptimized
    );
    assert_eq!(
        selected.budget_receipt().scope(),
        worth_store_budgets::PreExecutionBudgetScope::Foreground
    );
}

#[test]
fn deterministic_selection_denies_when_budget_is_exceeded_before_execution() {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let degraded = access_shapes()
        .explicit_degraded_exact_scan(DegradedExactScanRequest::new().with_budget_rows(10_000))
        .unwrap();

    let denial = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_read_request(
                    lifecycle,
                    crate::keyspace::admit_page_key(
                        key_domain,
                        worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
                        worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap(),
                    )
                    .expect("page identity must pass ordinary key admission"),
                    root_materialization(lifecycle, 11),
                    degraded,
                )
                .expect("test request must pass ordinary admission"),
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .unwrap_err();

    assert!(matches!(denial, AccessPlanSelectionDenied::BudgetDenied(_)));
}

#[test]
fn degraded_exact_scan_uses_explicit_rule_and_plan_bound_budget_receipt() {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let degraded = access_shapes()
        .explicit_degraded_exact_scan(DegradedExactScanRequest::new().with_budget_rows(8))
        .unwrap();

    let selected = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_read_request(
                    lifecycle,
                    crate::keyspace::admit_page_key(
                        key_domain,
                        worth_store_physical_format::PhysicalSegmentId::from_raw(1).unwrap(),
                        worth_store_physical_format::PhysicalPageId::from_raw(1).unwrap(),
                    )
                    .expect("page identity must pass ordinary key admission"),
                    root_materialization(lifecycle, 9),
                    degraded,
                )
                .expect("test request must pass ordinary admission"),
            PreExecutionBudgetEnvelope::terminal_default(),
        )
        .into_degraded()
        .expect("explicit degraded request must issue degraded scan authority");

    assert_eq!(
        selected.selection_rule(),
        DeterministicSelectionRule::ExplicitDegradedExactScan
    );
    assert_eq!(
        selected.primary_candidate().outcome(),
        &SelectionCandidateOutcome::Eligible(
            SelectionCandidateEligibility::ExplicitDegradedExactScan {
                planned_counter_envelope: selected.planned_counter_envelope(),
                budget_rows: 8,
            },
        )
    );
}

#[test]
fn deterministic_selection_denies_when_no_strategy_is_eligible() {
    let (lifecycle, key_domain) = admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalRootManifest,
        StoreKeyScope::StoreManagedRoot,
        StoreTenantScope::StoreInternal,
        StoreAuthenticityRequirement::not_required(),
        StoreCustodyPosture::InternalStoreCustody,
    );
    let access_shape = access_planning().range_access();

    let denial = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_read_request(
                    lifecycle,
                    crate::keyspace::admit_root_key(
                        key_domain,
                        worth_store_physical_format::PhysicalRootReference::from_raw(1).unwrap(),
                    )
                    .expect("root identity must pass ordinary key admission"),
                    root_materialization(lifecycle, 5),
                    access_shape,
                )
                .expect("test request must pass ordinary admission"),
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .unwrap_err();

    assert_eq!(denial, AccessPlanSelectionDenied::NoEligibleAlternative);
}
