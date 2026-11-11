pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::restored_replay::cross_process::*;

#[test]
fn byte_only_interim_checkpoint_from_another_process_refuses_without_capture_origin() {
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
    let Some(directory) = phase_directory() else {
        return;
    };
    let bytes = std::fs::read(directory.join("checkpoint")).expect("capture is read");
    let record = std::fs::read_to_string(directory.join("record")).expect("the record is read");
    let [_commit, _ordinal] = record
        .split(' ')
        .map(|part| part.parse::<u64>().expect("the record holds numbers"))
        .collect::<Vec<_>>()[..]
    else {
        panic!("the record holds the commit and the ordinal");
    };
    let denial = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .err()
    .expect("a file of bytes carries no capture-issued original authority");
    let worth_query_host::facade::application_installation::WorthQueryApplicationPreparationDenial::Graph(denial) = denial else {
        panic!("expected the typed missing-origin refusal: {denial:?}");
    };
    assert_eq!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::InterimCheckpointCaptureOriginMissing);
}
