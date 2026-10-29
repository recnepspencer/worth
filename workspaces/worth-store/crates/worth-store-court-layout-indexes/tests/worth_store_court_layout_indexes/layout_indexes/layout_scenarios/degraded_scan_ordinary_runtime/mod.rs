use {crate::degraded_scan_ordinary_runtime::physical_fixture::admitted_catalog, crate::degraded_scan_ordinary_runtime::physical_fixture::advanced_catalog, crate::degraded_scan_ordinary_runtime::physical_fixture::foreign_store_identity, crate::degraded_scan_ordinary_runtime::physical_fixture::open_runtime, crate::degraded_scan_ordinary_runtime::physical_fixture::open_runtime_for_store};
use {worth_store_budgets::PreExecutionBudgetEnvelope};
use {worth_store_layout_indexes::layout_degraded_scan_runtime, worth_store_layout_indexes::DegradedExactScanExecutionRequest, worth_store_layout_indexes::DegradedScanReadinessView, worth_store_layout_indexes::PlannedCounterObservation};
use {worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalSegmentId};
use {worth_store_security::admitted_tenant_page_security_scope_for_layout_partition_test};

#[test]
fn ordinary_degraded_scan_facade_admits_selects_and_executes() {
    let catalog = admitted_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let mut physical = open_runtime();
    physical
        .publish_physical_root()
        .expect("degraded scan fixture requires a published physical root");

    let execution = layout_degraded_scan_runtime()
        .execute(
            worth_store_layout_indexes::DegradedExactScanExecutionRequest::new(
                &catalog,
                security.witnesses(),
                worth_store_physical_format::PhysicalSegmentId::from_raw(7).unwrap(),
                worth_store_physical_format::PhysicalPageId::from_raw(3).unwrap(),
                8,
                worth_store_budgets::PreExecutionBudgetEnvelope::terminal_default(),
            ),
            &mut physical,
        )
        .expect("ordinary declarations must execute through the degraded scan owner facade");

    assert_eq!(execution.observed_rows(), 1);
    assert_eq!(
        execution.physical_observation().scan().counters().scans(),
        1
    );
    assert!(execution.physical_observation().scan().is_budget_exact());
    assert_eq!(
        execution
            .physical_observation()
            .scan()
            .request()
            .budget_rows(),
        8
    );
    assert_eq!(
        execution.current_materialization().materialization(),
        execution.selected().materialization()
    );
    let receipt = execution.counter_receipt();
    assert_eq!(
        receipt.observation(),
        worth_store_layout_indexes::PlannedCounterObservation::WithinEnvelope
    );
    assert_eq!(receipt.observed().allocation_events(), 3);
    assert_eq!(receipt.planned().range_steps(), 8);
    assert_eq!(receipt.observed().range_steps(), 1);
}

#[test]
fn ordinary_degraded_scan_rebinds_to_the_observed_catalog_frontier() {
    let catalog = admitted_catalog();
    let advanced = advanced_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let runtime = layout_degraded_scan_runtime();
    let request = |catalog| {
        worth_store_layout_indexes::DegradedExactScanExecutionRequest::new(
            catalog,
            security.witnesses(),
            worth_store_physical_format::PhysicalSegmentId::from_raw(7).unwrap(),
            worth_store_physical_format::PhysicalPageId::from_raw(3).unwrap(),
            8,
            worth_store_budgets::PreExecutionBudgetEnvelope::terminal_default(),
        )
    };

    let stale = runtime
        .prepare(request(&catalog).against_current_catalog(&advanced))
        .expect("ordinary planning must classify execution-time freshness");
    assert!(matches!(stale.view(), worth_store_layout_indexes::DegradedScanReadinessView::Stale(_)));
    let stale = stale
        .into_stale()
        .expect("advanced catalog must displace the original plan");
    let stale_plan = stale.selected().fingerprint().clone();

    let ready = runtime
        .rebind(stale, request(&advanced))
        .expect("replacement planning at the observed frontier must rebind");
    let trace = ready.rebind_trace().expect("rebind must retain lineage");
    assert_eq!(trace.stale_plan(), &stale_plan);
    assert_ne!(trace.stale_plan(), trace.replacement_plan());

    let mut physical = open_runtime();
    physical.publish_physical_root().unwrap();
    let execution = runtime
        .execute_ready(ready, &mut physical)
        .expect("rebound readiness must execute through the physical owner");
    assert_eq!(execution.observed_rows(), 1);
}

#[test]
fn equal_shaped_physical_scan_from_another_store_is_rejected() {
    let catalog = admitted_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let foreign_identity = foreign_store_identity();
    let mut physical = open_runtime_for_store(&foreign_identity);
    physical.publish_physical_root().unwrap();

    let denial = layout_degraded_scan_runtime()
        .execute(
            worth_store_layout_indexes::DegradedExactScanExecutionRequest::new(
                &catalog,
                security.witnesses(),
                worth_store_physical_format::PhysicalSegmentId::from_raw(7).unwrap(),
                worth_store_physical_format::PhysicalPageId::from_raw(3).unwrap(),
                8,
                worth_store_budgets::PreExecutionBudgetEnvelope::terminal_default(),
            ),
            &mut physical,
        )
        .unwrap_err();

    assert!(matches!(
        denial,
        worth_store_layout_indexes::DegradedExactScanExecutionDenied::Physical(
            worth_store_layout_indexes::PhysicalDegradedExecutionDenial::StoreAuthorityMismatch { .. }
        )
    ));
}

#[test]
fn broad_scan_is_budget_denied_before_physical_execution() {
    let catalog = admitted_catalog();
    let security = admitted_tenant_page_security_scope_for_layout_partition_test();
    let mut physical = open_runtime();
    physical.publish_physical_root().unwrap();
    let counters_before = physical.counters();

    let denial = layout_degraded_scan_runtime()
        .execute(
            worth_store_layout_indexes::DegradedExactScanExecutionRequest::new(
                &catalog,
                security.witnesses(),
                worth_store_physical_format::PhysicalSegmentId::from_raw(7).unwrap(),
                worth_store_physical_format::PhysicalPageId::from_raw(3).unwrap(),
                10_000,
                worth_store_budgets::PreExecutionBudgetEnvelope::foreground_default(),
            ),
            &mut physical,
        )
        .unwrap_err();

    assert!(matches!(
        denial,
        worth_store_layout_indexes::DegradedExactScanExecutionDenied::Selection(
            worth_store_layout_indexes::AccessPlanSelectionDenied::BudgetDenied(_)
        )
    ));
    assert_eq!(
        physical.counters(),
        counters_before,
        "budget rejection must precede physical scan work"
    );
}

