use super::super::lane::WorkflowCertificationLane;
use crate::harness::fixtures::{execution_preflights, preview_bridge::active_preview_artifacts};
use crate::preview::{
    admit_authoritative_preview_comparison_candidate, admit_preview_promotion_parity_comparison,
    admit_promotion_eligible_preview_session_plan_binding, bind_preflight_to_preview_session,
    execute_promotion_eligible_preview_session_plan, PreviewEvaluationClass,
    PreviewSessionQueryContext,
};
use crate::workflow::MergeLoweringInput;
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, build_workflow_replay_bundle,
    lower_merge_workflow_declaration, lower_query_writeback_declaration,
    shape_merge_authority_outcome, WorkflowAuthorityTargetFamily, WorkflowBindingSource,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowDeclarationRequest,
    WorkflowFreshnessPolicy, WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;

pub(super) fn merge_lowering_lane() -> WorkflowCertificationLane {
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
            WorkflowFreshnessPolicy::AllowExplicitRebind,
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
    let replay = build_workflow_replay_bundle(&outcome);

    let mut lane = WorkflowCertificationLane::from_declaration(&declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.lowered_freshness_binding = Some(lowered.freshness_binding().as_str().to_string());
    lane.authority_outcome_family = Some(outcome.family().as_str().to_string());
    lane.replay_bundle_digest = Some(replay.bundle_digest().to_string());
    lane.result_digest = outcome.authoritative_outcome_digest().to_string();
    lane.delivery_digest = replay.delivery_or_failure_digest().to_string();
    lane.prediction_drift_outcome = Some(outcome.prediction_drift_outcome().as_str().to_string());
    lane.budget_outcome = Some(outcome.budget_outcome().as_str().to_string());
    lane.predicted_declaration_width =
        Some(outcome.prediction_report().predicted_declaration_width());
    lane.predicted_lowering_width = Some(outcome.prediction_report().predicted_lowering_width());
    lane.realized_width = Some(outcome.realized_width());
    lane.lowering_width = Some(lowered.counters().workflow_lowering_width());
    lane.lowering_executor_rediscovery_count =
        Some(lowered.counters().workflow_executor_rediscovery_count());
    lane.replay_executor_rediscovery_count =
        Some(replay.counters().workflow_executor_rediscovery_count());
    lane
}

pub(super) fn writeback_lowering_lane() -> WorkflowCertificationLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
        .expect("runtime binding should admit");
    let declaration = admit_query_workflow_declaration(
        &binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::WritebackLoweringNarrow,
            WorkflowAuthorityTargetFamily::BridgeWriteback,
            WorkflowCostClass::WritebackLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::ExactBasis,
        ),
    )
    .expect("runtime writeback declaration should admit");
    let lowered = lower_query_writeback_declaration(
        &declaration,
        WritebackLoweringInput::projected_state_diff(),
    )
    .expect("writeback lowering should succeed");
    let outcome = crate::workflow::shape_writeback_authority_outcome(&lowered);
    let replay = build_workflow_replay_bundle(&outcome);

    let mut lane = WorkflowCertificationLane::from_declaration(&declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.lowered_freshness_binding = Some(lowered.freshness_binding().as_str().to_string());
    lane.authority_outcome_family = Some(outcome.family().as_str().to_string());
    lane.replay_bundle_digest = Some(replay.bundle_digest().to_string());
    lane.result_digest = outcome.authoritative_outcome_digest().to_string();
    lane.delivery_digest = replay.delivery_or_failure_digest().to_string();
    lane.prediction_drift_outcome = Some(outcome.prediction_drift_outcome().as_str().to_string());
    lane.budget_outcome = Some(outcome.budget_outcome().as_str().to_string());
    lane.predicted_declaration_width =
        Some(outcome.prediction_report().predicted_declaration_width());
    lane.predicted_lowering_width = Some(outcome.prediction_report().predicted_lowering_width());
    lane.realized_width = Some(outcome.realized_width());
    lane.lowering_width = Some(lowered.counters().workflow_lowering_width());
    lane.lowering_executor_rediscovery_count =
        Some(lowered.counters().workflow_executor_rediscovery_count());
    lane.replay_executor_rediscovery_count =
        Some(replay.counters().workflow_executor_rediscovery_count());
    lane
}

pub(super) fn preview_merge_lowering_lane() -> WorkflowCertificationLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("workflow-certification-preview-merge");
    let binding = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("preview merge binding should succeed");
    let execution = execute_promotion_eligible_preview_session_plan(
        &admit_promotion_eligible_preview_session_plan_binding(binding)
            .expect("promotion binding should admit"),
    )
    .expect("promotion execution should succeed");
    let candidate_execution = crate::execution::execute_preflight_bundle(&preflight)
        .expect("candidate execution should succeed");
    let candidate =
        admit_authoritative_preview_comparison_candidate(&preflight, &candidate_execution)
            .expect("candidate should admit");
    let comparison = admit_preview_promotion_parity_comparison(&execution, &candidate)
        .expect("promotion parity should admit");
    let workflow_binding = bind_workflow_context(
        WorkflowBindingSource::PreviewPromotionComparison(&comparison),
    )
    .expect("promotion comparison should bind");
    let declaration = admit_query_workflow_declaration(
        &workflow_binding,
        WorkflowDeclarationRequest::new(
            WorkflowDeclarationFamily::MergeLoweringNarrow,
            WorkflowAuthorityTargetFamily::RelationalMerge,
            WorkflowCostClass::MergeLoweringNarrow,
            WorkflowBudgetClass::AuthorityTargetBounded,
            WorkflowFreshnessPolicy::AllowExplicitRebind,
        ),
    )
    .expect("preview merge declaration should admit");
    let lowered = lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("preview merge lowering should succeed");

    let mut lane = WorkflowCertificationLane::from_declaration(&declaration);
    lane.lowered_request_digest = Some(lowered.lowering_for_reporting().to_string());
    lane.lowered_freshness_binding = Some(lowered.freshness_binding().as_str().to_string());
    lane.result_digest = lowered.lowering_for_reporting().to_string();
    lane.delivery_digest = lowered.lowering_for_reporting().to_string();
    lane.lowering_width = Some(lowered.counters().workflow_lowering_width());
    lane.lowering_executor_rediscovery_count =
        Some(lowered.counters().workflow_executor_rediscovery_count());
    lane
}
