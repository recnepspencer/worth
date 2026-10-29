pub(crate) use worth_query_testkit_query::facade::query::suite::operation_native_access_matrix::collection_window::*;

#[test]
fn window_resolution_cost_is_independent_of_total_collection_width() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut small = matrix_workspace(request_execution, "collection-window-scale-small", 4, false);
    let mut large = matrix_workspace(
        request_execution,
        "collection-window-scale-large",
        512,
        true,
    );
    let (small_collection, _) = bound_collection(&mut small);
    let (large_collection, _) = bound_collection(&mut large);
    let small_window = first_window(&small_collection, 3);
    let large_window = first_window(&large_collection, 3);

    assert_eq!(small_window.counters(), large_window.counters());
    assert_eq!(large_window.counters().rows_visited, 3);
    assert_eq!(large_window.counters().ordered_index_probes, 1);
    assert_eq!(large_window.counters().unrelated_rows_scanned, 0);
}
