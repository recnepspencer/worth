pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_primary_backend::*;

#[test]
fn restored_primary_backend_resumes_the_captured_ledger_position() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let directory = unique_directory("restored-primary-backend");
    let host = open_directory_on_first_program(directory);
    let captured = ledger_position(
        request_execution,
        &host.granular_invalidation_installation(),
    );
    let directory = host.checkpoint_and_close_for_reopen();

    let restored = open_directory_on_first_program(directory);
    assert_eq!(
        ledger_position(
            request_execution,
            &restored.granular_invalidation_installation()
        ),
        captured,
        "a reopened Directory Home answers the ledger position close held"
    );
}
