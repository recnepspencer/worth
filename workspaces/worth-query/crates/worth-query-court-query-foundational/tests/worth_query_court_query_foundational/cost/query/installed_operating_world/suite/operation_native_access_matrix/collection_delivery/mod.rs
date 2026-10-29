pub(crate) use worth_query_testkit_query::facade::query::suite::operation_native_access_matrix::collection_delivery::*;

#[test]
fn patch_cost_is_independent_of_total_collection_width() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let small = bounded_patch_counters(request_execution, "collection-patch-scale-small", 4);
    let large = bounded_patch_counters(request_execution, "collection-patch-scale-large", 512);

    assert_eq!(small, large);
    assert_eq!(large.entity_point_lookups, 1);
    assert_eq!(large.ordering_index_updates, 1);
    assert_eq!(large.fresh_window_rows_visited, 7);
    assert_eq!(large.full_collection_scans, 0);
    assert_eq!(large.unrelated_consumer_scans, 0);
}
