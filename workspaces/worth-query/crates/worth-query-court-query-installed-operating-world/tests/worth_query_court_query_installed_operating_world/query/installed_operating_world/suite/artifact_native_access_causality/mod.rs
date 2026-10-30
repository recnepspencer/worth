pub(crate) use worth_query_testkit_query::facade::query::suite::artifact_native_access_causality::*;

#[test]
fn native_physical_counters_move_only_with_their_declared_causal_axes() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bulk = execute(request_execution, "native-bulk");
    let field = execute(request_execution, "native-field");
    let chunks = execute(request_execution, "native-short-chunks");
    let scalar = execute(request_execution, "native-scalar");
    let projection_small = execute(request_execution, "native-projection-small");
    let projection_wide = execute(request_execution, "native-projection-wide");

    assert_same_two_field_payload(&bulk, &chunks);
    assert_same_two_field_payload(&bulk, &scalar);
    assert_eq!(bulk.evidence().counters().rows_exposed, ROWS);
    assert_eq!(chunks.evidence().counters().rows_exposed, ROWS);
    assert_eq!(scalar.evidence().counters().rows_exposed, 0);
    assert_eq!(bulk.evidence().counters().provider_contacts, 1);
    assert_eq!(bulk.evidence().counters().row_batch_contacts, 1);
    assert_eq!(bulk.evidence().counters().chunk_contacts, 0);
    assert_eq!(bulk.evidence().counters().scalar_calls, 0);
    assert_eq!(chunks.evidence().counters().provider_contacts, 12);
    assert_eq!(chunks.evidence().counters().row_batch_contacts, 11);
    assert_eq!(chunks.evidence().counters().chunk_contacts, 11);
    assert_eq!(chunks.evidence().counters().scalar_calls, 0);
    assert_eq!(scalar.evidence().counters().provider_contacts, ROWS * 2);
    assert_eq!(scalar.evidence().counters().row_batch_contacts, 0);
    assert_eq!(scalar.evidence().counters().chunk_contacts, 0);
    assert_eq!(scalar.evidence().counters().scalar_calls, ROWS * 2);

    let field_counters = field.evidence().counters();
    assert_eq!(field_counters.rows_exposed, ROWS);
    assert_eq!(field_counters.values_exposed, ROWS);
    assert_eq!(field_counters.source_bytes, ROWS * 8);
    assert_eq!(field_counters.provider_contacts, 1);
    assert_eq!(field_counters.field_slice_contacts, 1);
    assert_eq!(field_counters.row_batch_contacts, 0);
    assert_eq!(bulk.evidence().counters().values_exposed, ROWS * 2);
    assert_eq!(bulk.evidence().counters().source_bytes, TWO_FIELD_BYTES);

    assert_projection_chunk_axis(&projection_small, &projection_wide);
    for success in [
        &bulk,
        &field,
        &chunks,
        &scalar,
        &projection_small,
        &projection_wide,
    ] {
        assert_eq!(success.evidence().counters().generic_row_clones, 0);
    }
}
