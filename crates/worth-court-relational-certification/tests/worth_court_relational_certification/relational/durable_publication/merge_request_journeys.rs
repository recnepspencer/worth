use super::merge_proof_journey::*;
use worth_relational::facade::{history::*, merge::*};
use worth_testkit_relational::durable_medium::TestDurableMedium;
#[test]
fn normalized_merge_request_equivalence_is_exact_across_authoring_lanes() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let planning_request = planning_request();
    let execution_request = execution_request();

    let normalized_from_planning = runtime
        .merge()
        .normalize_merge_planning_request(
            runtime
                .bind_merge_planning_request(planning_request.clone())
                .unwrap(),
        )
        .expect("normalized planning request");
    let normalized_from_execution = runtime
        .merge()
        .normalize_merge_request(
            runtime
                .bind_merge_execution_request(execution_request.clone())
                .unwrap(),
        )
        .expect("normalized execution request");
    let explicit_specialist = NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .expect("specialist explicit request");

    assert_eq!(normalized_from_planning, normalized_from_execution);
    assert_eq!(normalized_from_planning, explicit_specialist);
    assert_eq!(
        normalized_from_planning.request_digest(),
        normalized_from_execution.request_digest()
    );
    assert_eq!(
        normalized_from_planning.request_digest(),
        explicit_specialist.request_digest()
    );
    assert_eq!(
        normalized_from_planning.family(),
        RelationalMergeRequestFamily::FullBranchReconciliation
    );
    assert_eq!(
        normalized_from_planning.scope(),
        RelationalMergeScope::FullBranch
    );
    assert_eq!(
        normalized_from_planning.correspondence_posture(),
        RelationalMergeCorrespondencePosture::Advisory
    );
    assert_eq!(
        normalized_from_planning.schema_reconciliation_posture(),
        RelationalMergeSchemaReconciliationPosture::Participate
    );
    assert_eq!(
        normalized_from_planning.topology_intent(),
        RelationalMergeTopologyIntent::PreserveTopologySemantics
    );
}
#[test]
fn planning_execution_and_publication_consume_normalized_request_authority() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = merge_ready_runtime(execution, &medium);
    let planning_request = planning_request();
    let execution_request = execution_request();
    let normalized = runtime
        .merge()
        .normalize_merge_request(
            runtime
                .bind_merge_execution_request(execution_request.clone())
                .unwrap(),
        )
        .expect("normalized request");

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(planning_request)
                .unwrap(),
        )
        .expect("planning artifact");
    assert_eq!(artifact.request, normalized);
    assert_eq!(
        artifact.request.request_digest(),
        normalized.request_digest()
    );
    assert_eq!(
        artifact.digest_basis.request.target_branch,
        normalized.target_branch().clone()
    );
    assert_eq!(
        artifact.digest_basis.request.source_branch,
        normalized.source_branch().clone()
    );
    assert_eq!(
        artifact.digest_basis.request.correspondence_posture,
        normalized.correspondence_posture()
    );
    assert_eq!(
        artifact.digest_basis.request.schema_reconciliation_posture,
        normalized.schema_reconciliation_posture()
    );
    assert_eq!(
        artifact.digest_basis.request.topology_intent,
        normalized.topology_intent()
    );
    assert_eq!(artifact.inspection_input().request(), &normalized);

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(execution_request)
                .unwrap(),
        )
        .expect("prepared merge");
    assert_eq!(prepared.request(), &normalized);
    assert_eq!(prepared.artifact().request, normalized);
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    assert_eq!(outcome.execution_summary.request, normalized);
    assert_eq!(
        outcome.execution_summary.request.request_digest(),
        normalized.request_digest()
    );
}
fn planning_request() -> MergePlanningRequest {
    MergePlanningRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}
fn execution_request() -> MergeExecutionRequest {
    MergeExecutionRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    )
}
