pub(crate) use worth_query_worlds::facade::certification::application_graph::restored_replay::cross_process::*;

#[test]
fn a_key_committed_in_another_process_resolves_by_its_durable_record() {
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
    let host = publish_on_first_program();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
        set_retention(&host, host.current_world(), RETENTION, KEY)
            .expect("the first request settles")
    else {
        panic!("the first request commits");
    };
    let checkpoint = host
        .with_host_advancement(|host_phase| {
            host.runtime()
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the host captures")
        })
        .expect("installed host call opens");
    std::fs::write(directory.join("checkpoint"), checkpoint.bytes()).expect("capture is written");
    let commit = receipt.committed_changes().commit_reference().commit_id.0;
    let ordinal = binding_runtime_ordinal(&host);
    std::fs::write(directory.join("record"), format!("{commit} {ordinal}"))
        .expect("the record is written");
}

#[test]
fn restore_phase() {
    let host_request =
        worth_query_worlds::facade::certification::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let Some(directory) = phase_directory() else {
        return;
    };
    let bytes = std::fs::read(directory.join("checkpoint")).expect("capture is read");
    let record = std::fs::read_to_string(directory.join("record")).expect("the record is read");
    let [commit, ordinal] = record
        .split(' ')
        .map(|part| part.parse::<u64>().expect("the record holds numbers"))
        .collect::<Vec<_>>()[..]
    else {
        panic!("the record holds the commit and the ordinal");
    };
    let restored = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_eq!(
        binding_runtime_ordinal(&restored),
        ordinal,
        "a fresh process mints the capturing process's runtime ordinal again"
    );
    assert_the_key_resolves_by_its_durable_record(request_execution, &restored, CommitId(commit));
}
