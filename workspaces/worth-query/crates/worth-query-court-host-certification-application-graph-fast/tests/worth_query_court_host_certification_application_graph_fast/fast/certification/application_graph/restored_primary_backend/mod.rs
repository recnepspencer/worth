pub(crate) use worth_query_worlds::facade::certification::application_graph::restored_primary_backend::*;

#[test]
fn the_first_production_write_after_restore_answers_as_before_the_capture() {
    let host_request =
        worth_query_worlds::facade::certification::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let before = write_twice(
        request_execution,
        &host.granular_invalidation_installation(),
    );
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
