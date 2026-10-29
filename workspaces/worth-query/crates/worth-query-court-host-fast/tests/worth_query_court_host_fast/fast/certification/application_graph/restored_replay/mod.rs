pub(crate) use worth_query_worlds::facade::certification::application_graph::restored_replay::*;

#[test]
fn a_key_committed_before_capture_keeps_its_intent_after_restore() {
    let host_request =
        worth_query_worlds::facade::certification::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let host = publish_on_first_program();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: original, ..
    } = set_retention(&host, host.current_world(), RETENTION, KEY)
        .expect("the first request settles")
    else {
        panic!("the first request commits");
    };
    let WorthQueryApplicationMutationOutcome::AlreadyCommitted(before) =
        set_retention(&host, host.current_world(), RETENTION, KEY)
            .expect("the retry before capture settles")
    else {
        panic!("the retry before capture replays");
    };
    assert!(before.is_same_authoritative_commit(&original));
    let commit = original.committed_changes().commit_reference().commit_id;
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
    assert_the_key_resolves_by_its_durable_record(request_execution, &restored, commit);
}
