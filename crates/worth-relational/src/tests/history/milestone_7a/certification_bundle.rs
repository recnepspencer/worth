use crate::facade::diagnostics::{DiagnosticCode, DiagnosticsScope};
use crate::facade::history::BranchId;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};
use crate::tests::support::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParentListSerializationArtifact {
    pub(super) root_commit_id: u64,
    pub(super) root_parents: Vec<u64>,
    pub(super) linear_commit_id: u64,
    pub(super) linear_parents: Vec<u64>,
    pub(super) feature_commit_id: u64,
    pub(super) feature_parents: Vec<u64>,
    pub(super) merge_ready_commit_id: u64,
    pub(super) merge_ready_parents: Vec<u64>,
    pub(super) replayed_merge_ready_parents: Vec<u64>,
    pub(super) recovered_merge_ready_parents: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AncestryQueryMatrix {
    pub(super) pre_merge_common_ancestor_commit_id: Option<u64>,
    pub(super) post_merge_common_ancestor_commit_id: Option<u64>,
    pub(super) merge_ready_commit_ancestor_closure: Vec<u64>,
    pub(super) feature_head_ancestor_closure: Vec<u64>,
    pub(super) main_head_commit_id: u64,
    pub(super) main_head_ancestor_closure: Vec<u64>,
    pub(super) inspected_merge_base_commit_id: Option<u64>,
    pub(super) feature_only_commit_closure: Vec<u64>,
    pub(super) main_only_commit_closure: Vec<u64>,
    pub(super) can_merge_feature_into_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MergeReadyHistoryCertificationBundle {
    pub(super) parent_list_serialization: ParentListSerializationArtifact,
    pub(super) ancestry_query_matrix: AncestryQueryMatrix,
    pub(super) replay_acceptance: ReplayAcceptanceEvidence,
    pub(super) durability_parity: DurabilityParityEvidence,
    pub(super) diagnostics: PublicationDiagnosticsEvidence,
    pub(super) branch_reasoning: BranchReasoningEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReplayAcceptanceEvidence {
    pub(super) failure_absent: bool,
    pub(super) reconstructed_closure_len: usize,
    pub(super) parents_match_publication: bool,
    pub(super) mismatches_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DurabilityParityEvidence {
    pub(super) recovered_parents_match_publication: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PublicationDiagnosticsEvidence {
    pub(super) merge_commit_published: bool,
    pub(super) merge_base_resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BranchReasoningEvidence {
    pub(super) inspected_merge_base_present: bool,
    pub(super) main_head_closure_contains_head: bool,
}

pub(super) fn run_merge_ready_history_shape_certification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> MergeReadyHistoryCertificationBundle {
    let runtime = persisted_runtime_with_test_schema();
    let root = create_entity_outcome(&runtime, "root");
    let linear = create_entity_outcome(&runtime, "linear");
    create_branch_from_main(&runtime, "feature");
    let feature =
        create_entity_outcome_on_branch(&runtime, "feature", BranchId("feature".to_string()));

    let pre_merge_common_ancestor = runtime
        .history()
        .latest_common_ancestor_between_branches(
            &BranchId("main".to_string()),
            &BranchId("feature".to_string()),
        )
        .map(|commit| commit.0);

    let merge = merge_commit_from_branches(
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let post_merge_main = create_entity_outcome(&runtime, "main-post-merge");
    let replay = replay_merge_ready_commit(execution, &runtime, merge.commit.commit_id);

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, persisted_runtime_with_test_schema);
    let post_merge_inspection = recovered.history().inspect_merge(
        &BranchId("feature".to_string()),
        &BranchId("main".to_string()),
    );

    let parent_list_serialization = ParentListSerializationArtifact {
        root_commit_id: root.commit.commit_id.0,
        root_parents: authoritative_parent_ids(&runtime, root.commit.commit_id),
        linear_commit_id: linear.commit.commit_id.0,
        linear_parents: authoritative_parent_ids(&runtime, linear.commit.commit_id),
        feature_commit_id: feature.commit.commit_id.0,
        feature_parents: authoritative_parent_ids(&runtime, feature.commit.commit_id),
        merge_ready_commit_id: merge.commit.commit_id.0,
        merge_ready_parents: authoritative_parent_ids(&runtime, merge.commit.commit_id),
        replayed_merge_ready_parents: replay
            .commit
            .as_ref()
            .map(|commit| commit_parent_ids(commit.ordered_parents().as_slice()))
            .unwrap_or_default(),
        recovered_merge_ready_parents: authoritative_parent_ids(&recovered, merge.commit.commit_id),
    };

    let ancestry_query_matrix = AncestryQueryMatrix {
        pre_merge_common_ancestor_commit_id: pre_merge_common_ancestor,
        post_merge_common_ancestor_commit_id: recovered
            .history()
            .latest_common_ancestor_between_branches(
                &BranchId("main".to_string()),
                &BranchId("feature".to_string()),
            )
            .map(|commit| commit.0),
        merge_ready_commit_ancestor_closure: commit_closure_ids(&recovered, merge.commit.commit_id),
        feature_head_ancestor_closure: commit_closure_ids(&recovered, feature.commit.commit_id),
        main_head_commit_id: post_merge_main.commit.commit_id.0,
        main_head_ancestor_closure: commit_closure_ids(
            &recovered,
            post_merge_main.commit.commit_id,
        ),
        inspected_merge_base_commit_id: post_merge_inspection.merge_base.map(|commit| commit.0),
        feature_only_commit_closure: post_merge_inspection
            .source_only_commits
            .iter()
            .map(|commit| commit.0)
            .collect(),
        main_only_commit_closure: post_merge_inspection
            .target_only_commits
            .iter()
            .map(|commit| commit.0)
            .collect(),
        can_merge_feature_into_main: post_merge_inspection.can_merge,
    };

    MergeReadyHistoryCertificationBundle {
        replay_acceptance: ReplayAcceptanceEvidence {
            failure_absent: replay.failure.is_none(),
            reconstructed_closure_len: replay.reconstructed_commit_closure.len(),
            parents_match_publication: replay
                .commit
                .as_ref()
                .map(|commit| commit_parent_ids(commit.ordered_parents().as_slice()))
                == Some(parent_list_serialization.merge_ready_parents.clone()),
            mismatches_empty: replay.mismatches.is_empty(),
        },
        durability_parity: DurabilityParityEvidence {
            recovered_parents_match_publication: parent_list_serialization
                .recovered_merge_ready_parents
                == parent_list_serialization.merge_ready_parents,
        },
        diagnostics: publication_diagnostics_evidence(&runtime),
        branch_reasoning: BranchReasoningEvidence {
            inspected_merge_base_present: ancestry_query_matrix
                .inspected_merge_base_commit_id
                .is_some(),
            main_head_closure_contains_head: ancestry_query_matrix
                .main_head_ancestor_closure
                .contains(&ancestry_query_matrix.main_head_commit_id),
        },
        parent_list_serialization,
        ancestry_query_matrix,
    }
}

fn authoritative_parent_ids(
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> Vec<u64> {
    commit_parent_ids(
        runtime
            .replay()
            .canonical_commit_envelope(commit_id)
            .unwrap()
            .commit
            .ordered_parents()
            .as_slice(),
    )
}

fn commit_parent_ids(parents: &[crate::facade::history::CommitId]) -> Vec<u64> {
    parents.iter().map(|parent| parent.0).collect()
}

fn commit_closure_ids(
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> Vec<u64> {
    runtime
        .history()
        .ancestor_closure_by_commit_id_order(commit_id)
        .into_iter()
        .map(|commit| commit.0)
        .collect()
}

fn replay_merge_ready_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: crate::facade::history::CommitId,
) -> crate::replay::data::RelationalReplayOutcome {
    runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        },
        execution,
    )
}

fn publication_diagnostics_evidence(runtime: &RelationalRuntime) -> PublicationDiagnosticsEvidence {
    let publication_diagnostic_codes = runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::PatchPublication)
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .filter(|entry| {
            matches!(
                entry.code,
                DiagnosticCode::MergeCommitPublished | DiagnosticCode::MergeBaseResolved
            )
        })
        .map(|entry| entry.code)
        .collect::<Vec<_>>();

    PublicationDiagnosticsEvidence {
        merge_commit_published: publication_diagnostic_codes
            .contains(&DiagnosticCode::MergeCommitPublished),
        merge_base_resolved: publication_diagnostic_codes
            .contains(&DiagnosticCode::MergeBaseResolved),
    }
}
