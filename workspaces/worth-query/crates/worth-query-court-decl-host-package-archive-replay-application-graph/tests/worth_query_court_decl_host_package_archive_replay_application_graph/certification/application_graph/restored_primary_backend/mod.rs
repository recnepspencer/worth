pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_primary_backend::*;

#[test]
fn restored_primary_backend_resumes_writes_from_the_same_captured_source() {
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
    // The backend authority deliberately declines after its Native commit.
    // Capture the genuine coherent Host source first, then compare backend
    // execution from that SAME source before and after its restoration.
    let before = write_twice(
        request_execution,
        &host.granular_invalidation_installation(),
    );
    assert_eq!(
        ledger_position(
            request_execution,
            &host.granular_invalidation_installation()
        ),
        captured.map(|position| PatchStreamPosition(position.0 + 2)),
        "each baseline backend write commits exactly once from the captured source"
    );
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    let installation = restored.granular_invalidation_installation();
    assert_eq!(
        ledger_position(request_execution, &installation),
        captured,
        "a restore answers the ledger position the capture held"
    );

    let after = write_twice(request_execution, &installation);
    assert_eq!(
        after, before,
        "the same write answers the same after restore"
    );
    assert_eq!(
        ledger_position(request_execution, &installation),
        captured.map(|position| PatchStreamPosition(position.0 + 2)),
        "each production write committed exactly once after the restore"
    );
}
