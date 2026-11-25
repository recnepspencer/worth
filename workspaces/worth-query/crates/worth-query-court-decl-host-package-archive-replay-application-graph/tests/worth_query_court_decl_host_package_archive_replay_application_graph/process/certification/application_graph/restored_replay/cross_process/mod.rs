use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_replay::cross_process::*;

#[test]
fn key_committed_in_one_process_resolves_after_directory_home_reopen_in_another_process() {
    let directory = std::env::temp_dir().join(format!(
        "worth-query-restored-replay-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).expect("the phase directory is created");
    run_phase("capture_phase", &directory);
    run_phase("restore_phase", &directory);
    std::fs::remove_dir_all(&directory).expect("the phase directory is removed");
}

#[test]
fn capture_phase() {
    let Some(directory) = phase_directory() else {
        return;
    };
    let host = open_directory_on_first_program(directory.join("home"));
    let (commit, ordinal) = {
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
            set_retention(&host, host.current_world(), RETENTION, KEY)
                .expect("the first request settles")
        else {
            panic!("the first request commits");
        };
        (
            receipt.committed_changes().commit_reference().commit_id.0,
            binding_runtime_ordinal(&host),
        )
    };
    let _ = host.checkpoint_and_close_for_reopen();
    std::fs::write(directory.join("record"), format!("{commit} {ordinal}"))
        .expect("the record is written");
}

#[test]
fn restore_phase() {
    let Some(directory) = phase_directory() else {
        return;
    };
    let host_request =
        worth_query_testkit_host::facade::host::certification_support::execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let record = std::fs::read_to_string(directory.join("record")).expect("the record is read");
    let [commit, _ordinal] = record
        .split(' ')
        .map(|part| part.parse::<u64>().expect("the record holds numbers"))
        .collect::<Vec<_>>()[..]
    else {
        panic!("the record holds the commit and the ordinal");
    };
    let restored = open_directory_on_first_program(directory.join("home"));
    assert_the_key_resolves_by_its_durable_record(request_execution, &restored, CommitId(commit));
    let _ = restored.close_for_reopen();
}
