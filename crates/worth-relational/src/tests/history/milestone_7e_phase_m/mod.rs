mod drift_support;
mod fixtures;

use drift_support::{
    branch_basis_drifted_authority, collaboration_family_drift, correspondence_drifted_authority,
    proof_packet_drifted_authority, schema_drifted_authority, strategy_drifted_authority,
};
use fixtures::{
    alternate_branch_basis, execute_feature_merge, published_merge_authority,
    runtime_with_collaboration_merge_history,
};

#[test]
fn merge_collaboration_truth_localizes_single_family_drift() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_collaboration_merge_history();
    let outcome = execute_feature_merge(execution, &runtime);
    let baseline = published_merge_authority(execution, &runtime, outcome.commit.commit.commit_id);

    assert_eq!(
        collaboration_family_drift(
            &baseline,
            &branch_basis_drifted_authority(&baseline, alternate_branch_basis(&runtime)),
        ),
        vec!["branch_basis"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &proof_packet_drifted_authority(&baseline)),
        vec!["proof_packet"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &correspondence_drifted_authority(&baseline)),
        vec!["correspondence_witness"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &schema_drifted_authority(&baseline)),
        vec!["schema_reconciliation_witness"]
    );
    assert_eq!(
        collaboration_family_drift(&baseline, &strategy_drifted_authority(&baseline)),
        vec!["strategy_witness"]
    );
}
