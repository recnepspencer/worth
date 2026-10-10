use super::*;

#[test]
fn replay_rebuild_denies_summary_and_packet_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime();
    let outcome = execute_merge(execution, &runtime);
    let mut envelope = runtime
        .replay()
        .canonical_commit_envelope(outcome.commit.commit.commit_id)
        .expect("canonical envelope");
    envelope
        .merge_execution_authority
        .as_mut()
        .expect("merge execution authority")
        .execution_summary
        .execution_digest = "forged-summary-digest".to_string();

    assert!(!envelope
        .merge_execution_authority
        .as_ref()
        .expect("merge execution authority")
        .retains_consistent_proof_packet_authority());

    envelope
        .merge_execution_authority
        .as_mut()
        .expect("merge execution authority")
        .execution_summary
        .request = crate::facade::merge::NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId("feature".to_string()),
        BranchId("main".to_string()),
        MergeIntent::ReconcileIntoTarget,
        crate::facade::merge::RelationalMergeCorrespondencePosture::Advisory,
        crate::facade::merge::RelationalMergeSchemaReconciliationPosture::Participate,
        crate::facade::merge::RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .expect("admitted normalized request");
    assert!(!envelope
        .merge_execution_authority
        .as_ref()
        .expect("merge execution authority")
        .retains_consistent_proof_packet_authority());
}
