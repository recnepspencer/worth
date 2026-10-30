pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::*;

#[test]
fn temporal_identity_work_is_cold_or_fresh_admission_only() {
    let host_request =
        worth_query_testkit_host::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    courtroom::temporal_identity_work_is_cold_or_fresh_admission_only(request_execution);
}

#[test]
fn reconstruction_work_scales_with_the_projection_not_unrelated_rows() {
    courtroom_lifecycle::reconstruction_work_ignores_unrelated_rows();
}
