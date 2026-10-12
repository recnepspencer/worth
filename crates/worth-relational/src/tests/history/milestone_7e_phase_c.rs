use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeExecutionRequest, MergeIntent, MergePlanningRequest, NormalizedRelationalMergeRequest,
    RelationalMergeCorrespondencePosture, RelationalMergeRequestFamily,
    RelationalMergeRequestNormalizationDenial, RelationalMergeSchemaReconciliationPosture,
    RelationalMergeScope, RelationalMergeTopologyIntent,
};
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    persisted_runtime_with_test_schema,
};

#[test]
fn normalized_merge_request_equivalence_is_exact_across_authoring_lanes() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let planning_request = planning_request();
    let execution_request = execution_request();

    let normalized_from_planning = runtime
        .merge()
        .normalize_merge_planning_request(planning_request.clone())
        .expect("normalized planning request");
    let normalized_from_execution = runtime
        .merge()
        .normalize_merge_request(execution_request.clone())
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
fn unsupported_request_posture_denies_before_history_or_planning_work() {
    assert_eq!(
        NormalizedRelationalMergeRequest::admit_full_branch(
            BranchId("main".to_string()),
            BranchId("feature".to_string()),
            MergeIntent::ReconcileIntoTarget,
            RelationalMergeCorrespondencePosture::Strict,
            RelationalMergeSchemaReconciliationPosture::Participate,
            RelationalMergeTopologyIntent::PreserveTopologySemantics,
        ),
        Err(
            RelationalMergeRequestNormalizationDenial::UnsupportedCorrespondencePosture {
                posture: RelationalMergeCorrespondencePosture::Strict,
            },
        )
    );
    assert_eq!(
        NormalizedRelationalMergeRequest::admit_full_branch(
            BranchId("main".to_string()),
            BranchId("feature".to_string()),
            MergeIntent::ReconcileIntoTarget,
            RelationalMergeCorrespondencePosture::Advisory,
            RelationalMergeSchemaReconciliationPosture::RequireCompatibility,
            RelationalMergeTopologyIntent::PreserveTopologySemantics,
        ),
        Err(
            RelationalMergeRequestNormalizationDenial::UnsupportedSchemaReconciliationPosture {
                posture: RelationalMergeSchemaReconciliationPosture::RequireCompatibility,
            },
        )
    );
    assert_eq!(
        NormalizedRelationalMergeRequest::admit_full_branch(
            BranchId("main".to_string()),
            BranchId("feature".to_string()),
            MergeIntent::ReconcileIntoTarget,
            RelationalMergeCorrespondencePosture::Advisory,
            RelationalMergeSchemaReconciliationPosture::Participate,
            RelationalMergeTopologyIntent::RequireStrictTopologyStability,
        ),
        Err(
            RelationalMergeRequestNormalizationDenial::UnsupportedTopologyIntent {
                intent: RelationalMergeTopologyIntent::RequireStrictTopologyStability,
            }
        )
    );
}

#[test]
fn planning_execution_and_publication_consume_normalized_request_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = merge_ready_runtime(execution);
    let planning_request = planning_request();
    let execution_request = execution_request();
    let normalized = runtime
        .merge()
        .normalize_merge_request(execution_request.clone())
        .expect("normalized request");

    let artifact = runtime
        .merge()
        .inspect_planning_scope(execution, planning_request)
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
        .prepare_merge_execution(execution, execution_request)
        .expect("prepared merge");
    assert_eq!(prepared.request(), &normalized);
    assert_eq!(prepared.artifact().request, normalized);
    assert_eq!(
        prepared.execution_ready_plan().request.request_digest(),
        normalized.request_digest()
    );
    assert_eq!(
        prepared.bound_executable_plan().authority_binding.request,
        normalized
    );

    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    assert_eq!(outcome.execution_summary.request, normalized);
    assert_eq!(
        outcome.execution_summary.request.request_digest(),
        normalized.request_digest()
    );
}

#[test]
fn normalized_request_deserialization_revalidates_admitted_truth() {
    let normalized = NormalizedRelationalMergeRequest::admit_full_branch(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
    )
    .expect("normalized request");
    let encoded = rmp_serde::to_vec(&normalized).expect("encode normalized request");
    let decoded: NormalizedRelationalMergeRequest =
        rmp_serde::from_slice(&encoded).expect("decode normalized request");
    assert_eq!(decoded, normalized);

    let forged = rmp_serde::to_vec_named(&serde_payload(
        "main",
        "feature",
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeRequestFamily::FullBranchReconciliation,
        RelationalMergeScope::FullBranch,
        RelationalMergeCorrespondencePosture::Advisory,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
        "forged-digest",
    ))
    .expect("encode forged payload");
    let forged_result: Result<NormalizedRelationalMergeRequest, _> = rmp_serde::from_slice(&forged);
    assert!(forged_result.is_err());

    let unsupported = rmp_serde::to_vec_named(&serde_payload(
        "main",
        "feature",
        MergeIntent::ReconcileIntoTarget,
        RelationalMergeRequestFamily::FullBranchReconciliation,
        RelationalMergeScope::FullBranch,
        RelationalMergeCorrespondencePosture::Strict,
        RelationalMergeSchemaReconciliationPosture::Participate,
        RelationalMergeTopologyIntent::PreserveTopologySemantics,
        normalized.request_digest(),
    ))
    .expect("encode unsupported payload");
    let unsupported_result: Result<NormalizedRelationalMergeRequest, _> =
        rmp_serde::from_slice(&unsupported);
    assert!(unsupported_result.is_err());
}

fn merge_ready_runtime(execution: &worth_execution::ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    runtime
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

#[derive(serde::Serialize)]
struct RequestSerdePayload<'a> {
    family: RelationalMergeRequestFamily,
    scope: RelationalMergeScope,
    target_branch: BranchId,
    source_branch: BranchId,
    merge_intent: MergeIntent,
    correspondence_posture: RelationalMergeCorrespondencePosture,
    schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
    topology_intent: RelationalMergeTopologyIntent,
    request_digest: &'a str,
}

fn serde_payload<'a>(
    target_branch: &'a str,
    source_branch: &'a str,
    merge_intent: MergeIntent,
    family: RelationalMergeRequestFamily,
    scope: RelationalMergeScope,
    correspondence_posture: RelationalMergeCorrespondencePosture,
    schema_reconciliation_posture: RelationalMergeSchemaReconciliationPosture,
    topology_intent: RelationalMergeTopologyIntent,
    request_digest: &'a str,
) -> RequestSerdePayload<'a> {
    RequestSerdePayload {
        family,
        scope,
        target_branch: BranchId(target_branch.to_string()),
        source_branch: BranchId(source_branch.to_string()),
        merge_intent,
        correspondence_posture,
        schema_reconciliation_posture,
        topology_intent,
        request_digest,
    }
}
