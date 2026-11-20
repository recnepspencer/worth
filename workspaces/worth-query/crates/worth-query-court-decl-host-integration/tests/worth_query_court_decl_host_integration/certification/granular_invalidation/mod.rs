pub(crate) use worth_query_testkit_certification_granular_invalidation::facade::certification::granular_invalidation::*;

#[test]
fn financial_curve_host_emits_real_granular_signal_delivery() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    financial_runtime_world::assert_financial_host_curve_delivery(request_execution);
}

#[test]
fn granular_transport_batches_merge_within_one_installation_in_either_order() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    delivery_convergence::assert_same_installation_transport_merge_in_both_orders(
        request_execution,
    );
}

#[test]
fn granular_transport_merge_refuses_a_foreign_installation_batch() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    delivery_convergence::assert_foreign_installation_merge_is_refused(request_execution);
}
