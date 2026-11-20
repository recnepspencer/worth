pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_primary_backend::*;

#[test]
fn restored_primary_backend_resumes_the_captured_ledger_position() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let captured = ledger_position(
        request_execution,
        &host.granular_invalidation_installation(),
    );
    let checkpoint = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the host captures")
        })
        .expect("installed host call opens");
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_eq!(
        ledger_position(
            request_execution,
            &restored.granular_invalidation_installation()
        ),
        captured,
        "a restore answers the ledger position the capture held"
    );
}
