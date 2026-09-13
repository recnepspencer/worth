mod absence;
mod owner_cases;

use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_layout_indexes::{
    layout_read_runtime, BTreeLookupExecutionView, BaselineBTreeLookupBranch,
    BaselineBTreeReadPreflight, BaselineBTreeReadShape, BaselineBTreeReadSource,
    LayoutReadAdmissionDenied, PageLookupRequest, PlannedCounterObservation,
};
use worth_store_physical_format::{PhysicalPageId, PhysicalSegmentId};
use worth_store_security::{
    admitted_store_managed_root_security_scope_for_layout_partition_test,
    admitted_tenant_page_security_scope_for_layout_partition_test,
};
use worth_store_test_support::harness::physical_isolation::epoch_scope as physical_support;
use worth_store_test_support::harness::physical_isolation::read_plan as plan_admission;
use worth_store_test_support::{
    admitted_layout_bootstrap_catalog, baseline_btree_probe_slot,
    deterministic_baseline_btree_read_preflight, deterministic_corrupt_leaf_btree_read_preflight,
    deterministic_cross_store_btree_read_preflight, deterministic_stale_child_btree_read_preflight,
    foreign_layout_physical_store_identity,
};
#[test]
fn ordinary_runtime_selects_and_executes_separator_directed_page_lookup() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let source = ordinary_source();
    let source_authority = source.store_authority_identity();
    let executed = layout_read_runtime()
        .execute_page_lookup(PageLookupRequest::new(
            &catalog,
            security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            source,
        ))
        .unwrap()
        .into_result()
        .unwrap();
    let BTreeLookupExecutionView::Found(lookup) = executed.view() else {
        panic!("ordinary probe must issue the found case")
    };

    assert_eq!(lookup.shape(), BaselineBTreeReadShape::PointLookup);
    assert_eq!(lookup.branch(), BaselineBTreeLookupBranch::Left);
    assert_eq!(lookup.probe_slot(), baseline_btree_probe_slot());
    let counters = lookup.exact_counters();
    assert_eq!(counters.point_lookups(), 1);
    assert_eq!(counters.range_lookups(), 0);
    assert_eq!(counters.page_touches(), 2);
    assert_eq!(counters.index_probes(), 2);
    assert_eq!(counters.key_comparisons(), 2);
    assert_eq!(counters.bytes_read(), 8_192);
    assert_eq!(counters.read_amplification(), 2);
    let receipt = executed.counter_receipt();
    assert_eq!(receipt.observation(), PlannedCounterObservation::Exact);
    assert_eq!(receipt.observed().allocation_events(), 4);
    assert_eq!(receipt.planned(), receipt.observed());
    assert_eq!(source_authority, security.witnesses().authority_identity());
    assert_eq!(
        executed
            .current_materialization()
            .materialization()
            .source()
            .btree_lookup_store_authority_identity(),
        Some(source_authority),
    );
    assert!(matches!(
        executed
            .current_materialization()
            .materialization()
            .source()
            .kind(),
        worth_store_layout_indexes::LayoutMaterializationSourceKind::BTreeRoot(_),
    ));
    assert_eq!(
        (*executed.stable_read())
            .read_plan_release()
            .protected_references_released(),
        3,
    );
}

#[test]
fn ordinary_runtime_derives_range_and_prefix_execution_from_admitted_intent() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let range = layout_read_runtime()
        .execute_page_lookup(PageLookupRequest::range(
            &catalog,
            security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            ordinary_source(),
        ))
        .unwrap()
        .into_result()
        .unwrap();
    let prefix = layout_read_runtime()
        .execute_page_lookup(PageLookupRequest::prefix(
            &catalog,
            security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            ordinary_source(),
        ))
        .unwrap()
        .into_result()
        .unwrap();

    let BTreeLookupExecutionView::Found(range) = range.view() else {
        panic!("range probe must issue the found case")
    };
    let BTreeLookupExecutionView::Found(prefix) = prefix.view() else {
        panic!("prefix probe must issue the found case")
    };
    assert_eq!(range.shape(), BaselineBTreeReadShape::RangeLookup);
    assert_eq!(range.exact_counters().range_steps(), 1);
    assert_eq!(prefix.shape(), BaselineBTreeReadShape::PrefixLookup);
    assert_eq!(prefix.exact_counters().prefix_steps(), 1);
}

#[test]
fn malformed_persisted_leaf_is_rejected_at_the_physical_read_transition() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let source = admit_source(deterministic_corrupt_leaf_btree_read_preflight());

    let outcome = layout_read_runtime()
        .execute_page_lookup(PageLookupRequest::new(
            &catalog,
            security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            source,
        ))
        .unwrap();
    assert!(matches!(
        outcome.view(),
        BTreeLookupExecutionView::Denied(
            worth_store_layout_indexes::BaselineBTreeExecutionDenial::InvalidLeafNode
        )
    ));
}

#[test]
fn stale_child_generation_cannot_read_newer_physical_bytes() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let source = admit_source(deterministic_stale_child_btree_read_preflight());
    let outcome = layout_read_runtime().execute_page_lookup(PageLookupRequest::new(
        &catalog,
        security.witnesses(),
        segment(7),
        page(9),
        baseline_btree_probe_slot(),
        PreExecutionBudgetEnvelope::foreground_default(),
        source,
    ));

    let outcome = outcome.unwrap();
    assert!(matches!(
        outcome.view(),
        BTreeLookupExecutionView::Denied(
            worth_store_layout_indexes::BaselineBTreeExecutionDenial::Physical(_)
        )
    ));
}

#[test]
fn stable_plan_omitting_a_candidate_child_cannot_issue_btree_source() {
    let preflight = deterministic_baseline_btree_read_preflight();
    let [root_reference, left_child, _right_child] = preflight.protected_references();
    let authority = physical_support::physical_authority_from_complete_closeout();
    let root = physical_support::current_root_from_authority(&authority);
    let references = plan_admission::protected_set([root_reference, left_child], 2);
    let incomplete = plan_admission::admit_plan(&authority, root, references, 8_192, 2);

    assert!(matches!(
        preflight.admit(incomplete),
        Err(worth_store_layout_indexes::BaselineBTreeExecutionDenial::StableReadPlan(
            worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::ExecutionTimeReferenceDiscovery,
        ))
    ));
}

#[test]
fn store_internal_root_scope_cannot_enter_tenant_page_lookup() {
    let catalog = admitted_layout_bootstrap_catalog();
    let wrong_security = admitted_store_managed_root_security_scope_for_layout_partition_test();
    let source = ordinary_source();

    assert_eq!(
        layout_read_runtime().execute_page_lookup(PageLookupRequest::new(
            &catalog,
            wrong_security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            source,
        )),
        Err(LayoutReadAdmissionDenied::SecurityScope),
    );
}

#[test]
fn equal_coordinate_btree_source_from_another_store_is_rejected_before_readiness() {
    let catalog = admitted_layout_bootstrap_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let ordinary = ordinary_source();
    let foreign_identity = foreign_layout_physical_store_identity();
    let foreign = admit_source_for_store(
        deterministic_cross_store_btree_read_preflight(),
        &foreign_identity,
    );

    assert_eq!(foreign.root_reference(), ordinary.root_reference());
    assert_ne!(
        foreign.store_authority_identity(),
        ordinary.store_authority_identity()
    );
    assert_eq!(
        layout_read_runtime().execute_page_lookup(PageLookupRequest::new(
            &catalog,
            security.witnesses(),
            segment(7),
            page(9),
            baseline_btree_probe_slot(),
            PreExecutionBudgetEnvelope::foreground_default(),
            foreign,
        )),
        Err(LayoutReadAdmissionDenied::ExactCoverage(
            worth_store_layout_indexes::MaterializationDenial::BTreeSourceStoreAuthorityMismatch,
        )),
    );
}

#[test]
fn equal_coordinate_stable_plan_from_another_store_cannot_admit_btree_source() {
    let preflight = deterministic_cross_store_btree_read_preflight();
    let ordinary_authority = physical_support::physical_authority_from_complete_closeout();
    let ordinary_root = physical_support::current_root_from_authority(&ordinary_authority);
    let references = plan_admission::protected_set(preflight.protected_references(), 3);
    let ordinary_plan =
        plan_admission::admit_plan(&ordinary_authority, ordinary_root, references, 12_288, 3);

    assert!(matches!(
        preflight.admit(ordinary_plan),
        Err(worth_store_layout_indexes::BaselineBTreeExecutionDenial::StableReadPlan(
            worth_store_physical_isolation::PhysicalReadPlanAdmissionDenial::StoreAuthorityMismatch,
        ))
    ));
}

fn segment(value: u64) -> PhysicalSegmentId {
    PhysicalSegmentId::from_raw(value).unwrap()
}

fn page(value: u64) -> PhysicalPageId {
    PhysicalPageId::from_raw(value).unwrap()
}

fn ordinary_source() -> BaselineBTreeReadSource {
    admit_source(deterministic_baseline_btree_read_preflight())
}

fn admit_source(preflight: BaselineBTreeReadPreflight) -> BaselineBTreeReadSource {
    let authority = physical_support::physical_authority_from_complete_closeout();
    admit_source_with_authority(preflight, &authority)
}

fn admit_source_for_store(
    preflight: BaselineBTreeReadPreflight,
    store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
) -> BaselineBTreeReadSource {
    let authority =
        physical_support::physical_authority_from_complete_closeout_for_store(store_identity);
    admit_source_with_authority(preflight, &authority)
}

fn admit_source_with_authority(
    preflight: BaselineBTreeReadPreflight,
    authority: &worth_store_physical_isolation::PhysicalReadStabilityAuthority,
) -> BaselineBTreeReadSource {
    let root = physical_support::current_root_from_authority(authority);
    let references = plan_admission::protected_set(preflight.protected_references(), 3);
    let plan = plan_admission::admit_plan(authority, root, references, 12_288, 3);
    preflight.admit(plan).unwrap()
}
