//! A stale prepared merge fails with a typed failure diagnostic and no published execution.
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use worth_relational::facade::{
    diagnostics::{DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope},
    history::BranchId,
    merge::{MergeExecutionError, MergeExecutionRequest, MergeIntent},
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};

#[test]
fn execute_prepared_merge_records_attempt_without_success_on_failure() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity_outcome(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    BranchId("main".to_string()),
                    BranchId("feature".to_string()),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .unwrap(),
        )
        .expect("prepared merge execution");
    create_entity_outcome(execution, &runtime, "head-drift");

    let error = runtime
        .execute_prepared_merge(execution, prepared)
        .expect_err("stale prepared merge should fail");

    assert!(matches!(error, MergeExecutionError::StaleBranchHead { .. }));
    assert!(runtime
        .publication()
        .diagnostics()
        .artifacts()
        .iter()
        .any(|artifact| artifact.scope == DiagnosticsScope::History
            && artifact.kind == DiagnosticsArtifactKind::Failure
            && artifact
                .entries
                .iter()
                .any(|entry| entry.code == DiagnosticCode::DeterministicMergeViolation)));
    assert!(!runtime
        .publication()
        .diagnostics()
        .artifacts()
        .iter()
        .any(|artifact| artifact.scope == DiagnosticsScope::History
            && artifact.kind == DiagnosticsArtifactKind::DetailedTrace
            && artifact
                .entries
                .iter()
                .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)));
}
