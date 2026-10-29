#[allow(unused_imports)]
pub(crate) use {crate::relational_certification::reference_context::root_cost_scale_axes::assert_flat_history_samples};

#[test]
#[ignore = "scheduled retained-root ceiling"]
fn selected_publication_cost_is_flat_through_documented_retention_ceiling() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    // The owner caps all simultaneously retained retired roots at 4,096, and
    // the fresh Court world honestly consumes part of that shared capacity.
    // The largest repeatable integer-ratio fixture is therefore 1/63/3,969;
    // it preserves a geometric slope without bypassing owner backpressure.
    assert_flat_history_samples(execution, &[1, 63, 3_969]);
}
