use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::facade::history::MergeConflictRecord;
use crate::facade::merge::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningError,
};
use crate::facade::transactions::CommitPhase;
use crate::tests::support::*;

#[test]
fn merge_commit_uses_deterministic_parent_order_and_advances_target_branch() {
    let runtime = runtime_with_test_schema();
    let main_outcome = create_entity_outcome(&runtime, "main-a");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_outcome =
        create_entity_outcome_on_branch(&runtime, "feature-a", BranchId("feature".to_string()));
    let merge_outcome = merge_commit_from_branches(
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );

    assert_eq!(
        merge_outcome.commit.parents,
        vec![
            main_outcome.commit.commit_id,
            feature_outcome.commit.commit_id
        ]
    );
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".to_string())),
        Some(merge_outcome.commit.clone())
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string())),
        Some(feature_outcome.commit.clone())
    );
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(merge_outcome.commit.commit_id)
        .unwrap();
    assert_eq!(
        envelope.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );
    assert_eq!(
        envelope.merge_base_commits,
        vec![main_outcome.commit.commit_id]
    );
    assert!(runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::PatchPublication)
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::MergeCommitPublished));
    assert!(runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::PatchPublication)
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::MergeBaseResolved));
    let publication_diagnostics = runtime.publication().diagnostics();
    let merge_diagnostic = publication_diagnostics
        .by_scope(DiagnosticsScope::PatchPublication)
        .into_iter()
        .flat_map(|artifact| artifact.entries.iter())
        .find(|entry| entry.code == DiagnosticCode::MergeCommitPublished)
        .expect("merge publication diagnostic");
    assert_eq!(
        diagnostic_field(merge_diagnostic, "history_shape"),
        &RelationalDiagnosticValue::String("MergeReady".to_string())
    );
    assert_eq!(
        diagnostic_field(merge_diagnostic, "parent_count"),
        &RelationalDiagnosticValue::Unsigned(2)
    );
    assert_eq!(
        diagnostic_field(merge_diagnostic, "authoritative_parent_list"),
        &RelationalDiagnosticValue::Array(vec![
            RelationalDiagnosticValue::CommitId(main_outcome.commit.commit_id),
            RelationalDiagnosticValue::CommitId(feature_outcome.commit.commit_id),
        ])
    );
}

#[test]
fn merge_commit_requires_existing_parent_branch_heads() {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "main-a");
    let error = runtime
        .prepare_merge_execution(MergeExecutionRequest::new(
            BranchId("main".to_string()),
            BranchId("missing".to_string()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        MergeExecutionPreparationError::Planning(MergePlanningError::MissingSourceHead { branch_id })
            if branch_id == BranchId("missing".to_string())
    ));
}

#[test]
fn merge_commit_rejects_stale_secondary_parent_binding_after_parent_moves() {
    let runtime = runtime_with_test_schema();
    create_entity_outcome(&runtime, "merge-base");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    create_entity_outcome_on_branch(
        &runtime,
        "feature-before-prepare",
        BranchId("feature".to_string()),
    );

    let target_identity = runtime.main_branch_identity();
    let target_options = runtime
        .transaction_validation_input_for(&target_identity)
        .expect("target binding is owner-issued");
    let parent_identity = runtime
        .branch_identity(&BranchId("feature".to_string()))
        .expect("parent identity is owner-issued");
    let parent_basis = runtime
        .transaction_validation_input_for(&parent_identity)
        .expect("parent basis is owner-issued")
        .basis()
        .clone();
    let prepared = {
        let transaction_validation_input =
            target_options.with_merge_parent_bases(vec![parent_basis]);
        runtime
            .begin_branch_transaction_with_owner_inputs(transaction_validation_input)
            .expect("owner-admitted transaction context")
    }
    .validate(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("target candidate validates before the parent moves");

    create_entity_outcome_on_branch(
        &runtime,
        "feature-after-prepare",
        BranchId("feature".to_string()),
    );
    let error = runtime
        .prepare_validated_proposal(prepared)
        .expect_err("a moved secondary parent must stale the prepared binding");
    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::InvalidMergeParent
    ));
}

#[test]
fn merge_inspection_reports_overlapping_authority() {
    let runtime = runtime_with_test_schema();
    let base = create_entity_outcome(&runtime, "shared");
    let shared = changed_entities(&base)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _main_update = update_entity(&runtime, shared, "main-updated");
    let _feature_update = update_entity_on_branch(
        &runtime,
        shared,
        "feature-updated",
        BranchId("feature".to_string()),
    );
    let inspection = runtime.history().inspect_merge(
        &BranchId("feature".to_string()),
        &BranchId("main".to_string()),
    );

    assert_eq!(inspection.merge_base, Some(base.commit.commit_id));
    assert!(!inspection.can_merge);
    assert_eq!(
        inspection.conflicting_records,
        vec![MergeConflictRecord::Entity(shared)]
    );
}

#[test]
fn merge_commit_rejects_overlapping_authority_since_merge_base() {
    let runtime = runtime_with_test_schema();
    let base = create_entity_outcome(&runtime, "shared");
    let shared = changed_entities(&base)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _main_update = update_entity(&runtime, shared, "main-updated");
    let _feature_update = update_entity_on_branch(
        &runtime,
        shared,
        "feature-updated",
        BranchId("feature".to_string()),
    );
    let txn = crate::tests::support::test_owner_begin_merge_transaction(
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let error = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error: ref conflict, .. }
            if conflict.code == DiagnosticCode::MergeConflictOverlap
    ));
    assert!(error.commit_log().has_rejection(
        CommitPhase::HistoryResolution,
        Some(DiagnosticCode::MergeConflictOverlap),
        None
    ));
    assert!(runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::History)
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::MergeConflictOverlap));
}
