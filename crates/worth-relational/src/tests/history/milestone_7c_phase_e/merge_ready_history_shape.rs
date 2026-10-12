use crate::facade::history::BranchId;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};

use super::fixtures::execute_feature_into_main_merge;

#[test]
fn execute_prepared_merge_produces_merge_ready_history_shape() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, merge, main_head_commit_id, feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("canonical merge envelope");
    let merge_authority = envelope
        .merge_execution_authority
        .expect("published merge execution authority");

    assert_eq!(
        runtime.history().latest_common_ancestor_between_branches(
            &BranchId("main".to_string()),
            &BranchId("feature".to_string())
        ),
        Some(feature_head_commit_id)
    );

    let inspection = runtime.history().inspect_merge(
        &BranchId("feature".to_string()),
        &BranchId("main".to_string()),
    );
    assert!(inspection.source_only_commits.is_empty());
    assert_eq!(inspection.merge_base, Some(feature_head_commit_id));
    assert_eq!(
        runtime
            .replay_authority()
            .replay_commit(
                RelationalReplayRequest {
                    commit_id: merge.commit.commit.commit_id,
                    branch_id: BranchId("main".to_string()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
                },
                execution
            )
            .unwrap()
            .commit
            .expect("replayed merge commit")
            .ordered_parents()
            .clone_inner(),
        merge.commit.commit.parents
    );
    assert_eq!(
        merge_authority.execution_summary.target_head_commit_id,
        merge.commit.commit.parents[0]
    );
    assert_eq!(
        merge_authority.execution_summary.source_head_commit_id,
        merge.commit.commit.parents[1]
    );
    assert_eq!(
        merge_authority
            .execution_summary
            .branch_basis
            .target_head()
            .commit_id,
        merge.commit.commit.parents[0]
    );
    assert_eq!(
        merge_authority
            .execution_summary
            .branch_basis
            .source_head()
            .commit_id,
        merge.commit.commit.parents[1]
    );
    assert_eq!(
        merge_authority.execution_summary.merge_base_commit_id,
        main_head_commit_id
    );
}
