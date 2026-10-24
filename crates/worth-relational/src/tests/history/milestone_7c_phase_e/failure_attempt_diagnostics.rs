use crate::facade::diagnostics::{DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope};
use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity_outcome, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

#[test]
fn execute_prepared_merge_records_attempt_without_success_on_failure() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
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
    create_entity_outcome(execution, &runtime, "head-drift");
    runtime.performance_access().reset_counters();

    let error = runtime
        .execute_prepared_merge(execution, prepared)
        .expect_err("stale prepared merge should fail");
    let counters = runtime.performance_access().counters();

    assert!(matches!(
        error,
        crate::facade::merge::MergeExecutionError::StaleBranchHead { .. }
    ));
    assert_eq!(counters.merge_execution_attempts, 1);
    assert_eq!(counters.merge_execution_requests, 0);
    assert_eq!(counters.merge_execution_records_admitted, 0);
    assert_eq!(counters.merge_execution_mutation_intents_emitted, 0);
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
