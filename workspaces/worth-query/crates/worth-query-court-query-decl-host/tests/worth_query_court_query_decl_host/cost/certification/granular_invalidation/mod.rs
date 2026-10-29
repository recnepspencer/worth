pub(crate) use worth_query_testkit_certification_granular_invalidation::facade::certification::granular_invalidation::*;

#[test]
fn implemented_structural_slopes_run_through_the_real_composition_root() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    structural_slopes::assert_measured_bridge_and_result_slopes(request_execution);
}
