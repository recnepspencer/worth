pub(crate) use worth_query_testkit_certification::facade::certification::temporal_conditional_oracle_parity::*;

#[test]
fn host_conditional_outcomes_follow_the_signal_class_across_matrix() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for case in Case::ALL {
        assert_case(request_execution, case);
    }
}

pub(crate) mod managed_cleanup;
