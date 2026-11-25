pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_replay::*;

#[test]
fn a_key_committed_before_capture_keeps_its_intent_after_restore() {
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let directory = unique_directory("restored-replay");
    let host = open_directory_on_first_program(directory);
    let commit = {
        let WorthQueryApplicationMutationOutcome::Committed {
            receipt: original, ..
        } = set_retention(&host, host.current_world(), RETENTION, KEY)
            .expect("the first request settles")
        else {
            panic!("the first request commits");
        };
        let WorthQueryApplicationMutationOutcome::RecoveredCommitted(before) =
            set_retention(&host, host.current_world(), RETENTION, KEY)
                .expect("the retry before close settles")
        else {
            panic!("the retry before close replays");
        };
        worth_query_testkit_certification_canonical::facade::recorded_receipt_comparison::assert_original(
            &before, &original,
        );
        original.committed_changes().commit_reference().commit_id
    };
    let directory = host.checkpoint_and_close_for_reopen();

    let restored = open_directory_on_first_program(directory);
    assert_the_key_resolves_by_its_durable_record(request_execution, &restored, commit);
}
