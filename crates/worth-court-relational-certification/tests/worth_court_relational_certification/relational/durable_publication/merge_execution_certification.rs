//! Machine-checkable merge-execution certification artifacts over public replay and durable recovery.
use super::branch_history_recovery::current_head;
use super::commit_meaning::durable_commit_meaning_matches;
use super::merge_execution_certification_cases::{
    exact_shared_merge_execution, prefer_richer_merge_execution,
    source_only_addition_merge_execution, MergeCertificationCase,
};
use super::merge_execution_observation::capture_aspect_truth_bundle;
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use super::merge_proof_journey::actual_basis;
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
use worth_relational::facade::runtime::{EntityReadRecord, RelationalRuntime};
use worth_testkit_relational::aspect_storage::definition::aspect_key;

#[derive(Debug, Clone, PartialEq, Eq)]
struct MergeExecutionCertificationArtifacts {
    merge_execution_digest: String,
    merge_execution_diagnostics_digest: String,
    visible_entity_count: usize,
    visible_relation_count: usize,
    replay_verified: bool,
    recovery_envelope_matches: bool,
    recovery_truth_matches: bool,
    branch_heads_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AuthoritativeMergeExecutionCertificationSuite {
    exact_shared: MergeExecutionCertificationArtifacts,
    source_only_addition: MergeExecutionCertificationArtifacts,
    prefer_richer_reconcile: MergeExecutionCertificationArtifacts,
}

#[test]
fn authoritative_merge_execution_certification_emits_machine_checkable_artifacts() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = AuthoritativeMergeExecutionCertificationSuite {
        exact_shared: certify_merge_execution_with_recovery(
            execution,
            &exact_shared_merge_execution(execution),
        ),
        source_only_addition: certify_merge_execution_with_recovery(
            execution,
            &source_only_addition_merge_execution(execution),
        ),
        prefer_richer_reconcile: certify_merge_execution_with_recovery(
            execution,
            &prefer_richer_merge_execution(execution),
        ),
    };

    for certification in [
        &suite.exact_shared,
        &suite.source_only_addition,
        &suite.prefer_richer_reconcile,
    ] {
        assert_machine_checkable_merge_execution_artifact(certification);
    }
}

fn assert_machine_checkable_merge_execution_artifact(
    certification: &MergeExecutionCertificationArtifacts,
) {
    assert!(certification.merge_execution_digest.len() > 8);
    assert!(certification.merge_execution_diagnostics_digest.len() > 8);
    assert!(certification.visible_entity_count > 0);
    assert_eq!(certification.visible_relation_count, 0);
    assert!(certification.replay_verified);
    assert!(certification.recovery_envelope_matches);
    assert!(certification.recovery_truth_matches);
    assert!(certification.branch_heads_match);
}

/// Certify one executed merge through public replay, then through a real
/// checkpoint and recovery of a receiver built over the case's own medium.
fn certify_merge_execution_with_recovery(
    execution: &ExecutionRequest<'_, '_>,
    case: &MergeCertificationCase,
) -> MergeExecutionCertificationArtifacts {
    let MergeCertificationCase {
        medium,
        declared_runtime,
        runtime,
        merge,
    } = case;
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("canonical merge envelope");
    let truth_bundle = capture_aspect_truth_bundle(execution, runtime, &[]);
    let (visible_entity_count, visible_relation_count) = visible_named_counts(execution, runtime);
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: merge.commit.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(
        replay.failure.is_none(),
        "replay certification failure: {replay:?}"
    );

    let (_recovery, recovered) =
        checkpoint_and_recover_durable(execution, runtime, medium, declared_runtime(medium));
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");
    let recovered_truth_bundle = capture_aspect_truth_bundle(execution, &recovered, &[]);

    let recovery_envelope_matches = durable_commit_meaning_matches(&envelope, &recovered_envelope);
    let recovery_truth_matches = truth_bundle.visible_truth == recovered_truth_bundle.visible_truth;
    assert!(recovery_envelope_matches);
    assert!(recovery_truth_matches);
    assert_eq!(
        merge_base_commit_id(runtime),
        merge_base_commit_id(&recovered)
    );
    let branch_heads_match = branch_head_commit_ids(runtime) == branch_head_commit_ids(&recovered);
    assert!(branch_heads_match);

    MergeExecutionCertificationArtifacts {
        merge_execution_digest: merge.execution_summary.execution_digest.clone(),
        merge_execution_diagnostics_digest: merge.execution_summary.diagnostics_digest.clone(),
        visible_entity_count,
        visible_relation_count,
        replay_verified: replay.failure.is_none(),
        recovery_envelope_matches,
        recovery_truth_matches,
        branch_heads_match,
    }
}

/// The owner-observed common ancestor of the actual feature and main heads.
fn merge_base_commit_id(runtime: &RelationalRuntime) -> CommitId {
    actual_basis(runtime, "feature", "main")
        .merge_base()
        .commit()
        .commit_id
}

fn branch_head_commit_ids(runtime: &RelationalRuntime) -> (Option<CommitId>, Option<CommitId>) {
    let head = |branch: &str| {
        current_head(runtime, &BranchId(branch.to_string())).map(|head| head.commit_id)
    };
    (head("main"), head("feature"))
}

/// How many named entities, and relations between named entities, the actual
/// main head shows: the sizes of the visible truth compared across recovery.
fn visible_named_counts(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (usize, usize) {
    let version_id = current_head(runtime, &BranchId("main".to_string()))
        .expect("branch head")
        .version_id;
    let read = runtime
        .read_truth()
        .try_read_version(execution, version_id)
        .expect("read request remains live")
        .expect("the actual branch head remains retained");
    let named = |record: &EntityReadRecord| {
        record
            .authoritative_aspect_state
            .as_ref()
            .is_some_and(|state| state.get(&aspect_key("name")).is_some())
    };
    let entities = read
        .entities()
        .iter()
        .filter(|entity| named(entity))
        .count();
    let relations = read
        .relations()
        .iter()
        .filter(|relation| {
            [relation.source, relation.target]
                .into_iter()
                .all(|endpoint| read.get_entity(endpoint).is_some_and(named))
        })
        .count();
    (entities, relations)
}
