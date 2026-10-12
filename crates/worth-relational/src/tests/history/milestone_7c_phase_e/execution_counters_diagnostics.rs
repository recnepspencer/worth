use crate::facade::diagnostics::{DiagnosticCode, DiagnosticsArtifactKind};
use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity_outcome, create_entity_outcome_on_branch,
    diagnostic_field_optional, persisted_runtime_with_test_schema,
};

#[test]
fn execute_prepared_merge_reports_execution_counters_and_structural_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
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
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    runtime.performance_access().reset_counters();

    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed prepared merge");
    let counters = runtime.performance_access().counters();

    assert_eq!(merge.structural_summary.executed_record_count, 1);
    assert_eq!(merge.structural_summary.adopted_source_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 1);
    assert_eq!(counters.merge_execution_attempts, 1);
    assert_eq!(counters.merge_execution_requests, 1);
    assert_eq!(counters.merge_execution_records_admitted, 1);
    assert_eq!(counters.merge_execution_mutation_intents_emitted, 1);
    assert_eq!(
        merge
            .commit
            .execution()
            .complexity_delta
            .merge_execution_attempts,
        1
    );
    assert_eq!(
        merge
            .commit
            .execution()
            .complexity_delta
            .merge_execution_requests,
        1
    );
    assert_eq!(
        merge
            .commit
            .execution()
            .complexity_delta
            .merge_execution_records_admitted,
        1
    );
    assert_eq!(
        merge
            .commit
            .execution()
            .complexity_delta
            .merge_execution_mutation_intents_emitted,
        1
    );
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
