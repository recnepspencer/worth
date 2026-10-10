use super::certification_bundle::run_merge_ready_history_shape_certification;

#[test]
fn merge_ready_history_replay_durability_and_diagnostics_are_authoritative() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let certification = run_merge_ready_history_shape_certification(execution);

    assert!(certification.replay_acceptance.failure_absent);
    assert_eq!(
        certification.replay_acceptance.reconstructed_closure_len,
        certification
            .ancestry_query_matrix
            .merge_ready_commit_ancestor_closure
            .len()
    );
    assert!(certification.replay_acceptance.parents_match_publication);
    assert!(certification.replay_acceptance.mismatches_empty);
    assert!(
        certification
            .durability_parity
            .recovered_parents_match_publication
    );
    assert!(certification.diagnostics.merge_commit_published);
    assert!(certification.diagnostics.merge_base_resolved);
}
