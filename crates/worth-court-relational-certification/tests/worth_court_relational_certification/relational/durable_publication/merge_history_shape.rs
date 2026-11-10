//! A published prepared merge leaves merge-ready ancestry, replay parents and basis authority.
use super::merge_feature_execution::execute_feature_into_main_merge;
use super::merge_proof_journey::actual_basis;
use worth_relational::facade::{
    history::BranchId,
    replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode},
};

#[test]
fn execute_prepared_merge_produces_merge_ready_history_shape() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (_medium, runtime, merge, main_head_commit_id, feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("canonical merge envelope");
    let merge_authority = envelope
        .merge_execution_authority
        .as_ref()
        .expect("published merge execution authority");

    // The owner-observed basis between the actual branch heads after the merge
    // selects the same common ancestor in either direction.
    assert_eq!(
        actual_basis(&runtime, "main", "feature")
            .merge_base()
            .commit()
            .commit_id,
        feature_head_commit_id
    );

    // Feature into main again: no source commit remains outside the target's
    // ancestry, and the merge base is the feature head itself.
    let postmerge_basis = actual_basis(&runtime, "feature", "main");
    let target_ancestors = postmerge_basis.merge_base().supporting_left_ancestors();
    assert!(postmerge_basis
        .merge_base()
        .supporting_right_ancestors()
        .iter()
        .all(|source_ancestor| target_ancestors.contains(source_ancestor)));
    assert_eq!(
        postmerge_basis.merge_base().commit().commit_id,
        feature_head_commit_id
    );
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
