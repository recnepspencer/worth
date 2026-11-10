use super::super::lane::WorkflowCertificationLane;
use crate::harness::fixtures::{
    execution_preflights,
    relational_merge_inspection::{
        deleted_vs_modified_inspection_artifact, source_addition_inspection_artifact,
        topology_region_conflict_inspection_artifact,
    },
};
use crate::workflow::MergeLoweringInput;
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, inspect_merge_conflicts,
    inspect_post_merge_outcome, lower_merge_workflow_declaration, shape_merge_authority_outcome,
    WorkflowAuthorityTargetFamily, WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
};
use worth_relational::facade::history::BranchId;

pub(super) fn conflict_inspection_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorkflowCertificationLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            WorkflowCostClass::MergeLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("runtime merge declaration should admit");
    let lowered = lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("merge lowering should succeed");
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::ConflictInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("conflict inspection declaration should admit");
    let inspection_artifact = source_addition_inspection_artifact(request_execution);
    let inspection =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &inspection_artifact)
            .expect("inspection should succeed");
    let source_addition_row = inspection
        .rows()
        .iter()
        .find(|row| row.merge_class() == "source_only_addition")
        .expect("source-only addition row should be present");

    let mut lane = WorkflowCertificationLane::from_declaration(&inspection_declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.inspection_family = Some(inspection.family().as_str().to_string());
    lane.result_digest = source_addition_row.conflict_scope_digest().to_string();
    lane.delivery_digest = inspection.declaration_digest().to_string();
    lane.prediction_drift_outcome = Some(inspection.drift_outcome().as_str().to_string());
    lane.predicted_declaration_width =
        Some(inspection.prediction_report().predicted_declaration_width());
    lane.predicted_inspection_width =
        Some(inspection.prediction_report().predicted_inspection_width());
    lane.inspection_row_width = Some(inspection.counters().workflow_inspection_row_width());
    lane.inspection_executor_rediscovery_count =
        Some(inspection.counters().workflow_executor_rediscovery_count());
    lane
}

pub(super) fn denied_conflict_inspection_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorkflowCertificationLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            WorkflowCostClass::MergeLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("runtime merge declaration should admit");
    let lowered = lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("merge lowering should succeed");
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::ConflictInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("conflict inspection declaration should admit");
    let inspection_artifact = deleted_vs_modified_inspection_artifact(request_execution);
    let topology_artifact = topology_region_conflict_inspection_artifact(request_execution);
    let inspection =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &inspection_artifact)
            .expect("inspection should succeed");
    let topology_inspection =
        inspect_merge_conflicts(&inspection_declaration, &lowered, &topology_artifact)
            .expect("topology inspection should succeed");
    let topology_row = topology_inspection
        .rows()
        .iter()
        .find(|row| row.merge_class() == "topology_region_conflict")
        .expect("topology-region row should be present");

    let mut lane = WorkflowCertificationLane::from_declaration(&inspection_declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.inspection_family = Some(inspection.family().as_str().to_string());
    lane.result_digest = topology_row.conflict_scope_digest().to_string();
    lane.delivery_digest = inspection.rows()[0].conflict_scope_digest().to_string();
    lane.prediction_drift_outcome = Some(inspection.drift_outcome().as_str().to_string());
    lane.predicted_declaration_width =
        Some(inspection.prediction_report().predicted_declaration_width());
    lane.predicted_inspection_width =
        Some(inspection.prediction_report().predicted_inspection_width());
    lane.inspection_row_width = Some(inspection.counters().workflow_inspection_row_width());
    lane.inspection_executor_rediscovery_count =
        Some(inspection.counters().workflow_executor_rediscovery_count());
    lane
}

pub(super) fn post_merge_inspection_lane() -> WorkflowCertificationLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            WorkflowCostClass::MergeLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("runtime merge declaration should admit");
    let lowered = lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("merge lowering should succeed");
    let outcome = shape_merge_authority_outcome(&lowered);
    let inspection_declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::PostMergeInspectionNarrow,
            WorkflowAuthorityTargetFamily::QueryInspection,
            WorkflowCostClass::InspectionNarrow,
            WorkflowBudgetClass::InspectionBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("post-merge inspection declaration should admit");
    let inspection = inspect_post_merge_outcome(&inspection_declaration, &outcome)
        .expect("inspection should succeed");

    let mut lane = WorkflowCertificationLane::from_declaration(&declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.authority_outcome_family = Some(outcome.family().as_str().to_string());
    lane.inspection_family = Some(inspection.family().as_str().to_string());
    lane.result_digest = inspection.rows()[0]
        .authoritative_commit_or_outcome_digest()
        .to_string();
    lane.delivery_digest = inspection.origin_digest().to_string();
    lane.prediction_drift_outcome = Some(inspection.drift_outcome().as_str().to_string());
    lane.predicted_declaration_width =
        Some(inspection.prediction_report().predicted_declaration_width());
    lane.predicted_inspection_width =
        Some(inspection.prediction_report().predicted_inspection_width());
    lane.inspection_row_width = Some(inspection.counters().workflow_inspection_row_width());
    lane.inspection_executor_rediscovery_count =
        Some(inspection.counters().workflow_executor_rediscovery_count());
    lane
}
