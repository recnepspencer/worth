use super::super::lane::{
    WorkflowCertificationLane, WorkflowCertificationRejection, WorkflowPerturbationClass,
};
use super::super::row_catalog::WorkflowRejectionRowSpec;
use super::runtime_lanes::{preview_foundation_lane, runtime_conflict_lane};
use crate::aspect_field_authoring::single_native_string_aspect_field_patch;
use crate::harness::certification::RejectionCertificationRow;
use crate::harness::fixtures::{execution_preflights, preview_bridge::active_preview_artifacts};
use crate::preview::{
    admit_preview_workflow_foundation, bind_preflight_to_preview_session,
    execute_read_only_preview_session_plan, PreviewEvaluationClass, PreviewSessionQueryContext,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, inspect_post_merge_outcome,
    lower_merge_workflow_declaration, lower_mutation_intent_declaration,
    lower_query_writeback_declaration, MergeLoweringInput, MutationLoweringInput,
    WorkflowAuthorityTargetFamily, WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
    WritebackLoweringInput,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};

pub(super) fn rejection_row(
    spec: &WorkflowRejectionRowSpec,
) -> RejectionCertificationRow<
    WorkflowPerturbationClass,
    WorkflowCertificationLane,
    WorkflowCertificationRejection,
> {
    let control_lane = runtime_conflict_lane();
    let parity_lane = preview_foundation_lane();
    let hostile_lane = match spec.row_name {
        "unsupported-workflow-family" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let (_runtime, active, execution_record) =
                active_preview_artifacts("workflow-certification-unsupported-post-merge");
            let preview_binding = bind_preflight_to_preview_session(
                preflight,
                PreviewSessionQueryContext::active(
                    &active,
                    &execution_record,
                    PreviewEvaluationClass::promotion_eligible(),
                ),
            )
            .expect("preview binding should succeed");
            let foundation =
                admit_preview_workflow_foundation(&preview_binding).expect("preview should admit");
            let workflow_binding =
                bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
                    .expect("preview workflow binding should admit");
            let error = admit_query_workflow_declaration(
                &workflow_binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::PostMergeInspectionNarrow,
                    WorkflowAuthorityTargetFamily::QueryInspection,
                    WorkflowCostClass::InspectionNarrow,
                    WorkflowBudgetClass::InspectionBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                ),
            )
            .expect_err("preview post-merge inspection should deny");
            WorkflowCertificationRejection::from_error(&error)
        }
        "invalid-basis-pairing" => {
            let preflight = execution_preflights::store_detail_preflight();
            let error = bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
                .expect_err("store-backed preflight should deny");
            WorkflowCertificationRejection::from_error(&error)
        }
        "preview-read-only-authority-request-forbidden" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let (_runtime, active, execution_record) =
                active_preview_artifacts("workflow-certification-read-only");
            let preview_binding = bind_preflight_to_preview_session(
                preflight,
                PreviewSessionQueryContext::active(
                    &active,
                    &execution_record,
                    PreviewEvaluationClass::read_only(),
                ),
            )
            .expect("read-only preview binding should succeed");
            let admitted_binding = crate::preview::admit_read_only_preview_session_plan_binding(
                preview_binding.clone(),
            )
            .expect("read-only binding should admit");
            let _execution = execute_read_only_preview_session_plan(&admitted_binding)
                .expect("read-only execution should succeed");
            let foundation = admit_preview_workflow_foundation(&preview_binding)
                .expect("read-only foundation should admit");
            let workflow_binding =
                bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
                    .expect("read-only workflow binding should admit");
            let error = admit_query_workflow_declaration(
                &workflow_binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::MergeLoweringNarrow,
                    WorkflowAuthorityTargetFamily::RelationalMerge,
                    WorkflowCostClass::MergeLoweringNarrow,
                    WorkflowBudgetClass::AuthorityTargetBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                ),
            )
            .expect_err("read-only preview merge intent should deny");
            WorkflowCertificationRejection::from_error(&error)
        }
        "unsupported-authority-target" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let binding =
                bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
                    .expect("runtime binding should admit");
            let error = admit_query_workflow_declaration(
                &binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::ConflictInspectionNarrow,
                    WorkflowAuthorityTargetFamily::BridgeWriteback,
                    WorkflowCostClass::InspectionNarrow,
                    WorkflowBudgetClass::InspectionBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                ),
            )
            .expect_err("unsupported authority target should deny");
            WorkflowCertificationRejection::from_error(&error)
        }
        "forbidden-workflow-broadening" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let binding =
                bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
                    .expect("runtime binding should admit");
            let error = admit_query_workflow_declaration(
                &binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::MergeLoweringNarrow,
                    WorkflowAuthorityTargetFamily::RelationalMerge,
                    WorkflowCostClass::MergeLoweringNarrow,
                    WorkflowBudgetClass::CrossBoundaryExpansion,
                    WorkflowFreshnessPolicy::AllowExplicitRebind,
                ),
            )
            .expect_err("cross-boundary expansion should deny");
            WorkflowCertificationRejection::from_error(&error)
        }
        "unsupported-merge-family" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let binding =
                bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
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
            .expect("writeback declaration should admit");
            let error = lower_merge_workflow_declaration(
                &declaration,
                MergeLoweringInput::reconcile_into_target(
                    BranchId("main".to_string()),
                    BranchId("candidate".to_string()),
                ),
            )
            .expect_err("writeback declaration should not lower as merge");
            WorkflowCertificationRejection::from_lowering_error(&error)
        }
        "unsupported-writeback-family" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let binding =
                bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
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
            .expect("merge declaration should admit");
            let error = lower_query_writeback_declaration(
                &declaration,
                WritebackLoweringInput::projected_state_diff(),
            )
            .expect_err("merge declaration should not lower as writeback");
            WorkflowCertificationRejection::from_lowering_error(&error)
        }
        "explicit-rebind-required" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let (_runtime, active, execution_record) =
                active_preview_artifacts("workflow-certification-writeback-rebind");
            let preview_binding = bind_preflight_to_preview_session(
                preflight,
                PreviewSessionQueryContext::active(
                    &active,
                    &execution_record,
                    PreviewEvaluationClass::promotion_eligible(),
                ),
            )
            .expect("preview binding should succeed");
            let foundation = admit_preview_workflow_foundation(&preview_binding)
                .expect("preview foundation should admit");
            let workflow_binding =
                bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
                    .expect("preview workflow binding should admit");
            let declaration = admit_query_workflow_declaration(
                &workflow_binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::WritebackLoweringNarrow,
                    WorkflowAuthorityTargetFamily::BridgeWriteback,
                    WorkflowCostClass::WritebackLoweringNarrow,
                    WorkflowBudgetClass::AuthorityTargetBounded,
                    WorkflowFreshnessPolicy::AllowExplicitRebind,
                ),
            )
            .expect("writeback declaration should admit");
            let error = lower_query_writeback_declaration(
                &declaration,
                WritebackLoweringInput::projected_state_diff(),
            )
            .expect_err("preview writeback should require rebind");
            WorkflowCertificationRejection::from_lowering_error(&error)
        }
        "stale-workflow-denied" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let (_runtime, active, execution_record) =
                active_preview_artifacts("workflow-certification-writeback-stale");
            let preview_binding = bind_preflight_to_preview_session(
                preflight,
                PreviewSessionQueryContext::active(
                    &active,
                    &execution_record,
                    PreviewEvaluationClass::promotion_eligible(),
                ),
            )
            .expect("preview binding should succeed");
            let foundation = admit_preview_workflow_foundation(&preview_binding)
                .expect("preview foundation should admit");
            let workflow_binding =
                bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
                    .expect("preview workflow binding should admit");
            let declaration = admit_query_workflow_declaration(
                &workflow_binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::WritebackLoweringNarrow,
                    WorkflowAuthorityTargetFamily::BridgeWriteback,
                    WorkflowCostClass::WritebackLoweringNarrow,
                    WorkflowBudgetClass::AuthorityTargetBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                ),
            )
            .expect("writeback declaration should admit");
            let error = lower_query_writeback_declaration(
                &declaration,
                WritebackLoweringInput::projected_state_diff(),
            )
            .expect_err("preview writeback with exact-basis freshness should stale-deny");
            WorkflowCertificationRejection::from_lowering_error(&error)
        }
        "post-merge-non-authoritative-outcome-forbidden" => {
            let preflight = execution_preflights::direct_runtime_preflight();
            let binding =
                bind_workflow_context(WorkflowBindingSource::RuntimePreflight(&preflight))
                    .expect("runtime binding should admit");
            let declaration = admit_query_workflow_declaration(
                &binding,
                WorkflowDeclarationRequest::new(
                    WorkflowDeclarationFamily::MutationLoweringNarrow,
                    WorkflowAuthorityTargetFamily::RelationalMutation,
                    WorkflowCostClass::MutationLoweringNarrow,
                    WorkflowBudgetClass::AuthorityTargetBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                ),
            )
            .expect("mutation declaration should admit");
            let authority_binding_identity = binding.basis_identity();
            let lowered = lower_mutation_intent_declaration(
                &declaration,
                authority_binding_identity,
                MutationLoweringInput::IntentReconciliation {
                    entity_id: EntityId::new(PartitionId(1), 41, 0),
                    desired_aspect_fields: single_native_string_aspect_field_patch(
                        "name", "name", "after",
                    )
                    .expect("name patch should be native"),
                },
            )
            .expect("mutation lowering should succeed");
            let outcome = crate::workflow::shape_mutation_authority_outcome(&lowered);
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
            let error = inspect_post_merge_outcome(&inspection_declaration, &outcome)
                .expect_err("mutation outcome should deny post-merge inspection");
            WorkflowCertificationRejection::from_inspection_error(&error)
        }
        other => panic!("unknown workflow rejection row {other}"),
    };

    RejectionCertificationRow {
        row_name: spec.row_name,
        perturbation_class: spec.perturbation_class,
        control_lane,
        hostile_lane,
        parity_lane,
    }
}
