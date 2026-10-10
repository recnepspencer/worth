use crate::facade::history::BranchId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{capture_aspect_truth_bundle, checkpoint_and_recover_with};

use super::artifacts::MergeExecutionCertificationArtifacts;

pub(super) fn certify_merge_execution_with_recovery<F>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    merge: &crate::facade::merge::MergeExecutionOutcome,
    recovered_factory: F,
) -> MergeExecutionCertificationArtifacts
where
    F: FnOnce() -> RelationalRuntime,
{
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("canonical merge envelope");
    let truth_bundle = capture_aspect_truth_bundle(execution, runtime, &[], &[], &[]);
    let direct_rebuild_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::AuditRecoveryVerification,
    );
    runtime
        .rebuild_runtime_from_plan(execution, direct_rebuild_plan)
        .unwrap_or_else(|error| panic!("direct replay rebuild failed: {error:?}"));
    let replay = runtime.replay_authority().replay_commit(
        crate::facade::replay::RelationalReplayRequest {
            commit_id: merge.commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: crate::facade::replay::ReplayExecutionMode::SerialDeterministic,
            verification_mode:
                crate::facade::replay::ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert!(
        replay.failure.is_none(),
        "replay certification failure: {replay:?}"
    );

    let (_recovery, recovered) = checkpoint_and_recover_with(execution, runtime, recovered_factory);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("recovered merge envelope");
    let recovered_truth_bundle = capture_aspect_truth_bundle(execution, &recovered, &[], &[], &[]);

    let recovery_envelope_matches = envelope == recovered_envelope;
    let recovery_truth_matches = truth_bundle.visible_truth == recovered_truth_bundle.visible_truth;
    assert!(recovery_envelope_matches);
    assert!(recovery_truth_matches);
    assert_eq!(
        runtime.history().latest_common_ancestor_between_branches(
            &BranchId("main".to_string()),
            &BranchId("feature".to_string())
        ),
        recovered.history().latest_common_ancestor_between_branches(
            &BranchId("main".to_string()),
            &BranchId("feature".to_string())
        )
    );
    let live_branch_heads = (
        runtime
            .history()
            .branch_head(&BranchId("main".to_string()))
            .map(|head| head.commit_id),
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string()))
            .map(|head| head.commit_id),
    );
    let recovered_branch_heads = (
        recovered
            .history()
            .branch_head(&BranchId("main".to_string()))
            .map(|head| head.commit_id),
        recovered
            .history()
            .branch_head(&BranchId("feature".to_string()))
            .map(|head| head.commit_id),
    );
    let branch_heads_match = live_branch_heads == recovered_branch_heads;
    assert!(branch_heads_match);

    MergeExecutionCertificationArtifacts {
        merge_execution_digest: merge.execution_summary.execution_digest.clone(),
        merge_execution_diagnostics_digest: merge.execution_summary.diagnostics_digest.clone(),
        visible_entity_count: truth_bundle.visible_truth.entity_names.len(),
        visible_relation_count: truth_bundle.visible_truth.relations.len(),
        replay_verified: replay.failure.is_none(),
        recovery_envelope_matches,
        recovery_truth_matches,
        branch_heads_match,
    }
}
