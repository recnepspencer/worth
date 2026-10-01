use std::collections::BTreeSet;

use worth_store_budgets::{PreExecutionBudgetEnvelope, PreExecutionBudgetScope};
use worth_store_contracts::{DurableArtifactFamilyId, WalRecordFamily};
use worth_store_physical_format::{PhysicalPageId, PhysicalRootReference, PhysicalSegmentId};
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

use super::access_request::AdmittedPlanningRequest;
use super::{
    access_plan_selection_cases, AccessPlanCostClass, AccessPlanCostEstimate,
    AccessPlanSelectionCaseId, AccessPlanSelectionView, AccessPlanSelector,
};
use crate::access::execution::AccessPathCounterSnapshot;
use crate::facade::access_planning;
use crate::strategy::tests_support::{
    admit_persisted_lsm_scope, admit_strategy_scope, persisted_lsm_materialization,
    root_manifest_scope,
};

#[test]
fn declared_selection_cases_equal_cases_emitted_by_ordinary_requests() {
    let (page_family, page_domain) = page_scope();
    let page_key = || {
        crate::keyspace::admit_page_key(
            page_domain,
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .expect("page identity must pass ordinary key admission")
    };
    let page_exact = page_materialization(page_family);
    let foreground = PreExecutionBudgetEnvelope::foreground_default();
    let maintenance = PreExecutionBudgetEnvelope::maintenance_default();
    let mut observed = BTreeSet::new();

    for shape in [
        access_planning().point_access(),
        access_planning().range_access(),
        access_planning().prefix_access(),
    ] {
        observed.insert(select_case(
            AccessPlanSelector
                .admit_read_request(page_family, page_key(), page_exact.clone(), shape)
                .expect("ordinary B-tree read request must admit"),
            foreground,
        ));
    }

    observed.insert(select_case(
        AccessPlanSelector
            .admit_recovery_request(
                page_family,
                page_key(),
                page_exact.clone(),
                access_planning()
                    .rebuild_access(crate::AccessLaneClassification::Maintenance)
                    .expect("B-tree replay shape must admit"),
            )
            .expect("ordinary B-tree replay request must admit"),
        maintenance,
    ));

    let (wal_family, wal_domain) = wal_scope();
    let wal_key = || {
        crate::keyspace::admit_wal_key(
            wal_domain,
            WalRecordFamily::DurableMutationIntent,
            worth_store_wal::StoreWalRecordIdentity::new(1),
        )
        .expect("WAL identity must pass ordinary key admission")
    };
    let wal_materialization = wal_materialization(wal_family);

    observed.insert(select_case(
        AccessPlanSelector
            .admit_read_request(
                wal_family,
                wal_key(),
                wal_materialization.clone(),
                access_planning().point_access(),
            )
            .expect("ordinary LSM lookup request must admit"),
        foreground,
    ));

    observed.insert(select_case(
        AccessPlanSelector
            .admit_read_request(
                page_family,
                page_key(),
                page_exact.clone(),
                crate::access_shapes()
                    .explicit_degraded_exact_scan(
                        crate::DegradedExactScanRequest::new()
                            .with_budget_rows(u16::MAX as u64 + 1),
                    )
                    .expect("unrepresentable degraded demand still admits its declaration"),
            )
            .expect("degraded cost denial must pass request admission"),
        PreExecutionBudgetEnvelope::new(
            PreExecutionBudgetScope::Terminal,
            u64::MAX,
            u16::MAX,
            u16::MAX,
            u16::MAX,
            u64::MAX,
        ),
    ));

    observed.insert(select_case(
        AccessPlanSelector
            .admit_read_request(
                wal_family,
                wal_key(),
                wal_materialization.clone(),
                access_planning().point_access(),
            )
            .expect("LSM budget denial must follow ordinary request admission"),
        PreExecutionBudgetEnvelope::new(
            PreExecutionBudgetScope::Foreground,
            0,
            0,
            0,
            0,
            0,
        ),
    ));
    observed.insert(select_case(
        AccessPlanSelector
            .admit_mutation_request(
                wal_family,
                wal_key(),
                crate::access_shapes()
                    .append(crate::PhysicalMutationShape::LogStructuredAppend)
                    .expect("LSM append shape must admit"),
            )
            .expect("ordinary LSM publication request must admit"),
        maintenance,
    ));
    observed.insert(select_case(
        AccessPlanSelector
            .admit_recovery_request(
                wal_family,
                wal_key(),
                wal_materialization.clone(),
                access_planning()
                    .rebuild_access(crate::AccessLaneClassification::Maintenance)
                    .expect("LSM replay shape must admit"),
            )
            .expect("ordinary LSM replay request must admit"),
        maintenance,
    ));
    observed.insert(select_case(
        AccessPlanSelector
            .admit_mutation_request(
                wal_family,
                wal_key(),
                crate::access_shapes()
                    .compaction_read(crate::PhysicalMutationShape::CompactionRewrite)
                    .expect("LSM compaction shape must admit"),
            )
            .expect("ordinary LSM compaction request must admit"),
        maintenance,
    ));

    observed.insert(select_case(
        AccessPlanSelector
            .admit_read_request(
                page_family,
                page_key(),
                page_exact.clone(),
                crate::access_shapes()
                    .explicit_degraded_exact_scan(
                        crate::DegradedExactScanRequest::new().with_budget_rows(8),
                    )
                    .expect("explicit degraded scan shape must admit"),
            )
            .expect("ordinary degraded request must admit"),
        PreExecutionBudgetEnvelope::terminal_default(),
    ));

    let (root_family, root_domain) = root_manifest_scope();
    observed.insert(select_case(
        AccessPlanSelector
            .admit_read_request(
                root_family,
                crate::keyspace::admit_root_key(
                    root_domain,
                    PhysicalRootReference::from_raw(1).unwrap(),
                )
                .expect("root identity must pass ordinary key admission"),
                page_materialization(root_family),
                access_planning().range_access(),
            )
            .expect("unsupported root request still passes request admission"),
        foreground,
    ));

    assert_eq!(
        observed,
        access_plan_selection_cases().collect(),
        "the decision owner must not advertise a case ordinary planning cannot emit",
    );
}

fn select_case<Request: AdmittedPlanningRequest>(
    request: Request,
    budget: PreExecutionBudgetEnvelope,
) -> AccessPlanSelectionCaseId {
    let outcome = AccessPlanSelector.select_admitted_with_budget(request, budget);
    match outcome.view() {
        AccessPlanSelectionView::LsmLookup(plan) => assert_operation_cost(
            plan.cost_estimate(),
            plan.budget_receipt(),
            AccessPlanCostClass::LsmLookup,
            plan.planned_counter_envelope().lookup(),
        ),
        AccessPlanSelectionView::LsmRunPublication(plan) => assert_operation_cost(
            plan.cost_estimate(),
            plan.budget_receipt(),
            AccessPlanCostClass::LsmRunPublication,
            plan.planned_counter_envelope().publication(),
        ),
        AccessPlanSelectionView::LsmReplayRecovery(plan) => assert_operation_cost(
            plan.cost_estimate(),
            plan.budget_receipt(),
            AccessPlanCostClass::LsmReplayRecovery,
            plan.planned_counter_envelope().recovery(),
        ),
        AccessPlanSelectionView::LsmCompaction(plan) => assert_operation_cost(
            plan.cost_estimate(),
            plan.budget_receipt(),
            AccessPlanCostClass::LsmCompaction,
            plan.planned_counter_envelope().publication(),
        ),
        AccessPlanSelectionView::Degraded(plan) => {
            let estimate = plan.cost_estimate();
            assert_eq!(estimate.class(), AccessPlanCostClass::DegradedExactScan);
            assert_eq!(
                estimate.operation_counters(),
                plan.planned_counter_envelope().lookup()
            );
            assert_eq!(estimate.estimated_range_touches(), 8);
            assert_eq!(estimate.estimated_byte_reads(), 8 * 64);
            assert_budget_binding(estimate, plan.budget_receipt());
        }
        AccessPlanSelectionView::Denied(_) => {}
    }
    outcome.case_id()
}

fn assert_operation_cost(
    estimate: &AccessPlanCostEstimate,
    receipt: worth_store_budgets::PreExecutionBudgetAdmissionReceipt,
    expected_class: AccessPlanCostClass,
    expected_counters: AccessPathCounterSnapshot,
) {
    assert_eq!(estimate.class(), expected_class);
    assert_eq!(estimate.operation_counters(), expected_counters);
    assert_eq!(
        estimate.estimated_page_reads(),
        expected_counters.page_touches()
    );
    assert_eq!(
        estimate.estimated_chunk_reads(),
        expected_counters.chunk_tree_node_reads()
    );
    assert_eq!(
        estimate.estimated_byte_reads(),
        expected_counters.bytes_read(),
        "non-degraded operation cost must use only its exact counter slice"
    );
    assert_budget_binding(estimate, receipt);
}

fn assert_budget_binding(
    estimate: &AccessPlanCostEstimate,
    receipt: worth_store_budgets::PreExecutionBudgetAdmissionReceipt,
) {
    let request = receipt.request();
    assert_eq!(
        request.estimated_memory_bytes(),
        estimate.estimated_memory_bytes()
    );
    assert_eq!(
        request.estimated_page_reads(),
        estimate.estimated_page_reads()
    );
    assert_eq!(
        request.estimated_chunk_reads(),
        estimate.estimated_chunk_reads()
    );
    assert_eq!(
        request.estimated_range_touches(),
        estimate.estimated_range_touches()
    );
    assert_eq!(
        request.estimated_byte_reads(),
        estimate.estimated_byte_reads()
    );
}

fn page_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    admit_strategy_scope(
        DurableArtifactFamilyId::PhysicalPage,
        StoreKeyScope::PageEnvelope,
        StoreTenantScope::TenantPhysicalBoundary,
        StoreAuthenticityRequirement::required(
            StoreAuthenticityRequirementClass::AuthenticatedFrame,
        ),
        StoreCustodyPosture::InternalStoreCustody,
    )
}

fn wal_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    admit_persisted_lsm_scope()
}

fn page_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    access_planning()
        .admit_current_catalog_root_materialization(family, &catalog)
        .expect("physical catalog must admit exact root materialization")
}

fn wal_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    persisted_lsm_materialization(family, &catalog).0
}
