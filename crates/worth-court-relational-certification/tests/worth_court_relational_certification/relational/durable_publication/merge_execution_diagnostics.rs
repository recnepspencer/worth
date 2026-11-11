//! A published prepared merge reports its structural summary and execution diagnostics.
use super::branch_history_publications::{create_entity_outcome_on_branch, fork};
use super::merge_execution_observation::diagnostic_field_optional;
use worth_relational::facade::{
    diagnostics::{DiagnosticCode, DiagnosticsArtifactKind},
    history::BranchId,
    merge::{MergeExecutionRequest, MergeIntent},
};
use worth_testkit_relational::{
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity_outcome, durable_runtime},
};

#[test]
fn execute_prepared_merge_reports_structural_summary_and_diagnostics() {
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

    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");

    assert_eq!(merge.structural_summary.executed_record_count, 1);
    assert_eq!(merge.structural_summary.adopted_source_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 1);
    assert!(merge.commit.diagnostics().iter().any(|artifact| {
        artifact.kind == DiagnosticsArtifactKind::MinimalSummary
            && artifact
                .entries
                .iter()
                .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
    }));
    let execution_artifact = merge
        .commit
        .diagnostics()
        .iter()
        .find(|artifact| {
            artifact.kind == DiagnosticsArtifactKind::DetailedTrace
                && artifact
                    .entries
                    .iter()
                    .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
        })
        .expect("merge execution detailed artifact");
    assert!(execution_artifact.entries.iter().all(|entry| {
        entry.code == DiagnosticCode::MergeExecutionPublished
            && diagnostic_field_optional(entry, "blocked_count").is_none()
            && diagnostic_field_optional(entry, "rejected_count").is_none()
    }));
}
