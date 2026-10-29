pub(crate) use worth_query_worlds::facade::query::suite::conditional_resource_lifecycle::*;

#[test]
fn selected_product_turnover_releases_query_and_signal_retention_together() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let node = conditional_node_contract::resource_lifecycle_node();
    let installation = installed_operation_fixture::conditional_installation(&node);
    let harness = PublicBridgeRuntimeHarness::new();
    let resources = runtime::WorthQueryConditionalExecutionResources::new(
        runtime::WorthQueryConditionalEvaluationCacheBudget::bounded(2, 1024 * 1024).unwrap(),
        SignalConditionalEvaluationBudget {
            maximum_retained_slots: Some(2),
            maximum_retained_bytes: Some(512 * 1024 * 1024),
            maximum_attempt_visits: None,
        },
    );
    let mut workspace = installed_operation_fixture::conditional_resource_workspace(
        request_execution,
        "conditional-resource-lifecycle",
        node,
        installation,
        &harness,
        resources,
    )
    .unwrap();
    let installed = workspace
        .domain(installed_operation_fixture::GeometryDomain)
        .unwrap();

    let first = create_sibling(&workspace, "conditional-resource-first", 1);
    execute_selected(&mut workspace, &installed, first);
    execute_default(&mut workspace, &installed);
    let full = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(full.query_retained_entries(), 2);
    assert_eq!(full.signal_retained_slots(), 2);
    assert!(full.query_retained_bytes() > 0);
    assert!(full.signal_retained_bytes() > 0);

    let reused = execute_selected(&mut workspace, &installed, first);
    let warm = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(warm.query_cache_hits(), full.query_cache_hits() + 1);
    assert_eq!(warm.query_cache_misses(), full.query_cache_misses());
    assert_eq!(warm.query_retained_bytes(), full.query_retained_bytes());
    assert_eq!(warm.signal_retained_bytes(), full.signal_retained_bytes());
    assert_eq!(reused.slot_reuse_hits, 1);
    assert_eq!(reused.source_admission_attempts, 0);
    assert_eq!(reused.compute_contacts, 0);

    let second = create_sibling(&workspace, "conditional-resource-second", 2);
    execute_selected(&mut workspace, &installed, second);
    let turned = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(turned.query_retained_entries(), 2);
    assert_eq!(turned.signal_retained_slots(), 2);
    assert_eq!(turned.query_cache_misses(), warm.query_cache_misses() + 1);
    assert_eq!(
        turned.query_cache_evictions(),
        warm.query_cache_evictions() + 1
    );
    assert_eq!(turned.query_retained_bytes(), warm.query_retained_bytes());
    assert_eq!(turned.signal_retained_bytes(), warm.signal_retained_bytes());

    execute_default(&mut workspace, &installed);
    let retried = workspace
        .conditional_evaluation_resource_observation()
        .unwrap();
    assert_eq!(retried.query_retained_entries(), 2);
    assert_eq!(retried.signal_retained_slots(), 2);
    assert_eq!(
        retried.query_cache_misses(),
        turned.query_cache_misses() + 1
    );
    assert_eq!(
        retried.query_cache_evictions(),
        turned.query_cache_evictions() + 1
    );
    assert_eq!(
        retried.query_retained_bytes(),
        turned.query_retained_bytes()
    );
    assert_eq!(
        retried.signal_retained_bytes(),
        turned.signal_retained_bytes()
    );
}
