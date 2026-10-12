use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::history::BranchId;
use crate::tests::support::{
    create_branch_from_main, create_entity_outcome, create_entity_outcome_on_branch,
    persisted_runtime_with_test_schema,
};

use super::fixtures::execute_feature_into_main_merge;

#[test]
fn execute_prepared_merge_publishes_ordered_multi_parent_commit_through_canonical_envelope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, merge, main_head_commit_id, feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let replay = runtime.replay();

    assert_eq!(merge.commit.merge_parent_count(), 1);
    assert_eq!(
        merge.commit.commit.parents,
        vec![main_head_commit_id, feature_head_commit_id]
    );
    assert_eq!(
        merge.execution_summary.target_head_commit_id,
        main_head_commit_id
    );
    assert_eq!(
        merge.execution_summary.source_head_commit_id,
        feature_head_commit_id
    );
    assert_eq!(merge.execution_summary.executed_record_count, 1);

    let envelope = replay
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("canonical merge envelope");
    assert_eq!(envelope.commit.parents, merge.commit.commit.parents);
    assert_eq!(
        envelope.merge_parent_branches,
        vec![BranchId("feature".to_string())]
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("main".to_string()))
            .expect("main branch head")
            .commit_id,
        merge.commit.commit.commit_id
    );
    assert_eq!(
        runtime
            .history()
            .branch_head(&BranchId("feature".to_string()))
            .expect("feature branch head")
            .commit_id,
        feature_head_commit_id
    );
}

#[test]
fn execute_prepared_merge_preserves_reserved_summary_when_optional_diagnostics_budget_is_zero() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = persisted_runtime_with_test_schema();
    runtime.configure_diagnostics_for_test(|profile| profile.max_entries_per_artifact = 0);
    create_entity_outcome(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            crate::facade::merge::MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: crate::facade::merge::MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");

    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    let replay = runtime.replay();
    let envelope = replay
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("canonical merge envelope");

    assert_eq!(envelope.diagnostics_summary.entries.len(), 1);
    assert_eq!(
        envelope.diagnostics_summary.entries[0].code,
        DiagnosticCode::MergeExecutionPublished
    );
}
