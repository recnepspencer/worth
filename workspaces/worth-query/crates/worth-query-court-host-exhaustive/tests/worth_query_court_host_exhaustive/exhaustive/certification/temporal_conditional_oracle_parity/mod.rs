pub(crate) use worth_query_worlds::facade::certification::temporal_conditional_oracle_parity::*;

pub(crate) mod managed_cleanup;

#[test]
fn host_and_internal_conditional_oracle_agree_across_phase_nine_matrix() {
    let host_request =
        worth_query_worlds::facade::certification::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for case in Case::ALL {
        assert_case(request_execution, case);
    }
}
