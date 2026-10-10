use super::support::*;
use crate::application::{
    WorthQueryConcurrentHostileMatrixPosture, WorthQueryConcurrentHostileMatrixSabotageKind,
};

#[test]
fn phase_sixteen_concurrent_hostile_matrix_closes_with_replay_stable_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = execute_phase_sixteen_concurrent_hostile_matrix(request_execution);

    assert_phase_sixteen_closed(&artifact);
    assert_eq!(
        artifact.posture(),
        WorthQueryConcurrentHostileMatrixPosture::Closed
    );
    assert!(!artifact.digest().as_str().is_empty());
    assert!(!artifact.replay_digest().is_empty());
}

#[test]
fn phase_sixteen_concurrent_hostile_matrix_uses_real_topology_and_runtime_counters() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = execute_phase_sixteen_concurrent_hostile_matrix(request_execution);
    let topology = artifact.topology();
    let counters = artifact.counters();

    assert_eq!(topology.reader_thread_count(), 3);
    assert_eq!(topology.submitter_thread_count(), 2);
    assert_eq!(topology.submission_round_count(), 3);
    assert_eq!(counters.committed_read_hot_path_lock_count(), 0);
    assert_eq!(counters.shared_read_mint_row_clone_count(), 0);
    assert_eq!(counters.reader_derived_evaluation_count(), 0);
    assert_eq!(counters.orphaned_snapshot_generation_count(), 0);
    assert_eq!(counters.unretired_read_pin_count(), 0);
    assert_eq!(counters.journal_gap_count(), 0);
    assert_eq!(counters.replay_residue_count(), 0);
    assert_eq!(counters.delivery_residue_count(), 0);
    assert!(counters.published_artifact_registry_lease_count() > 0);
}

#[test]
fn phase_sixteen_counter_sabotage_opens_the_matrix_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = execute_phase_sixteen_concurrent_hostile_matrix(request_execution);
    let sabotage = phase_sixteen_sabotage_proofs(&artifact);

    assert_eq!(sabotage.len(), 8);
    assert!(sabotage.iter().any(|proof| {
        proof.kind() == WorthQueryConcurrentHostileMatrixSabotageKind::JournalGap
    }));
    assert!(sabotage.iter().any(|proof| {
        proof.kind() == WorthQueryConcurrentHostileMatrixSabotageKind::DeliveryResidue
    }));
    assert!(sabotage.iter().all(|proof| proof.opens_posture()));
    assert!(sabotage
        .iter()
        .all(|proof| proof.posture_after_sabotage()
            != WorthQueryConcurrentHostileMatrixPosture::Closed));
    assert!(sabotage
        .iter()
        .all(|proof| proof.opened_counter_residue_count() > 0));
}
