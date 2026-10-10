use super::support::*;

#[test]
fn hostile_certification_artifact_matches_serialized_replay_and_repeated_runs() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = execute_runtime_hostile_schedule(request_execution);
    let replay = replay_runtime_hostile_schedule(request_execution);
    let second = execute_runtime_hostile_schedule(request_execution);

    assert_eq!(first, replay);
    assert_eq!(first, second);
    assert_eq!(first.digest().as_str(), replay.digest().as_str());
    assert_eq!(first.digest().as_str(), second.digest().as_str());
}

#[test]
fn hostile_certification_counters_prove_zero_shared_read_lock_and_phase_eleven_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = execute_runtime_hostile_schedule(request_execution);
    let counters = artifact.counters();

    assert_eq!(counters.committed_read_hot_path_lock_count(), 0);
    assert_eq!(counters.shared_read_mint_row_clone_count(), 0);
    assert!(counters.published_artifact_registry_lease_count() > 0);
    assert_eq!(counters.reader_derived_evaluation_count(), 0);
    assert_eq!(counters.orphaned_snapshot_generation_count(), 0);
    assert_eq!(counters.unretired_read_pin_count(), 0);
    assert_eq!(counters.journal_gap_count(), 0);
    assert_eq!(counters.delivery_residue_count(), 0);
}
