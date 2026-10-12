use worth_proof::TransitionOutcome;
use worth_relational::facade::history::BranchId;

use crate::harness::fixtures::execution_preflights;
use crate::harness::fixtures::preview_bridge::active_preview_artifacts;
use crate::harness::fixtures::relational_merge_inspection::source_addition_inspection_artifact;
use crate::preview::{
    admit_preview_workflow_foundation, bind_preflight_to_preview_session, PreviewEvaluationClass,
    PreviewSessionQueryContext,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context, lower_merge_workflow_declaration,
    lower_query_writeback_declaration, MergeLoweringInput, WorkflowAuthorityTargetFamily,
    WorkflowBindingSource, WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily,
    WorkflowDeclarationRequest, WorkflowFreshnessPolicy, WritebackLoweringInput,
};

use super::test_support::{admitted_plan_target_parts, declaration_target, ready, success};
use super::{
    materialize_query_conflict_inspection_artifact,
    materialize_query_post_merge_inspection_artifact,
    WorthQueryDomainCapabilityProgressionDenialKind, WorthQueryWorkflowContributionAuthoring,
    WorthQueryWorkflowContributionPayload, WorthQueryWorkflowContributionPosture,
    WorthQueryWorkflowInspectionSemantics, WorthQueryWorkflowRuntimeBindingSemantics,
    WorthQueryWorkflowRuntimeSemantics,
};

#[test]
fn workflow_conflict_inspection_materializer_builds_query_conflict_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let lowered_merge = runtime_lowered_merge();
    let artifact = success(materialize_query_conflict_inspection_artifact(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_merge_conflict_inspection(
            "spatial.workflow.conflict_inspection",
            "runtime merge inspection should materialize a query conflict artifact",
            lowered_merge,
            source_addition_inspection_artifact(request_execution),
        )
        .bind_to_declaration_target(declaration_target("intent-workflow-conflict-inspection")),
    )));

    assert_eq!(artifact.family().as_str(), "merge_workflow_narrow");
    assert!(!artifact.rows().is_empty());
    assert_eq!(artifact.counters().workflow_conflict_inspection_count(), 1);
}

#[test]
fn workflow_post_merge_inspection_materializer_builds_query_post_merge_artifact() {
    let lowered_merge = runtime_lowered_merge();
    let artifact = success(materialize_query_post_merge_inspection_artifact(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_post_merge_merge_outcome_inspection(
            "spatial.workflow.post_merge_inspection",
            "runtime merge outcome should materialize a post-merge inspection artifact",
            lowered_merge,
        )
        .bind_to_admitted_plan_target(admitted_plan_target_parts(
            "plan-workflow-post-merge-inspection",
            "request-post-merge",
            "eligibility-post-merge",
            "decision-post-merge",
        )),
    )));

    assert_eq!(artifact.family().as_str(), "authoritative_outcome_narrow");
    assert_eq!(artifact.rows().len(), 1);
    assert_eq!(
        artifact.counters().workflow_post_merge_inspection_count(),
        1
    );
}

#[test]
fn workflow_conflict_inspection_materializer_builds_preview_promotion_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let lowered_merge = preview_lowered_merge();
    let artifact = success(materialize_query_conflict_inspection_artifact(ready(
        WorthQueryWorkflowContributionAuthoring::promotion_eligible_merge_conflict_inspection(
            "spatial.workflow.preview_conflict_inspection",
            "preview promotion merge inspection should materialize a query conflict artifact",
            lowered_merge,
            source_addition_inspection_artifact(request_execution),
        )
        .bind_to_declaration_target(declaration_target(
            "intent-workflow-preview-conflict-inspection",
        )),
    )));

    assert_eq!(artifact.family().as_str(), "merge_workflow_narrow");
    assert_eq!(artifact.counters().workflow_conflict_inspection_count(), 1);
}

#[test]
fn workflow_post_merge_inspection_materializer_builds_writeback_artifact() {
    let lowered_writeback = runtime_lowered_writeback();
    let artifact = success(materialize_query_post_merge_inspection_artifact(ready(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_post_merge_writeback_outcome_inspection(
            "spatial.workflow.post_merge_writeback_inspection",
            "runtime writeback outcome should materialize a post-merge inspection artifact",
            lowered_writeback,
        )
        .bind_to_declaration_target(declaration_target(
            "intent-workflow-post-merge-writeback-inspection",
        )),
    )));

    assert_eq!(artifact.family().as_str(), "authoritative_outcome_narrow");
    assert_eq!(artifact.rows().len(), 1);
    assert_eq!(
        artifact.counters().workflow_post_merge_inspection_count(),
        1
    );
}

#[test]
fn workflow_inspection_materializer_denies_missing_inspection_semantics() {
    let denied = materialize_query_conflict_inspection_artifact(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-missing-inspection"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::ConfirmationRequired,
                "spatial.workflow.missing_inspection",
                "missing workflow inspection semantics should deny",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                        crate::memory_workspace::admit_external_snapshot_label(
                            "runtime-snapshot:inspection",
                        ),
                    ),
                    WorkflowDeclarationFamily::ConflictInspectionNarrow,
                    WorkflowAuthorityTargetFamily::QueryInspection,
                    WorkflowCostClass::InspectionNarrow,
                    WorkflowBudgetClass::InspectionBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                )),
            ),
        ),
    ));

    assert!(matches!(
        denied,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));
}

#[test]
fn workflow_post_merge_inspection_denies_preview_bound_outcomes() {
    let lowered_merge = preview_lowered_merge();
    let denied = materialize_query_post_merge_inspection_artifact(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-invalid-post-merge-preview"),
            WorthQueryWorkflowContributionPayload::with_runtime_and_inspection_semantics(
                WorthQueryWorkflowContributionPosture::PromotionEligible,
                "spatial.workflow.invalid_post_merge_preview",
                "preview merge artifacts should not admit post-merge inspection declarations",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:invalid-post-merge",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                    ),
                    WorkflowDeclarationFamily::PostMergeInspectionNarrow,
                    WorkflowAuthorityTargetFamily::QueryInspection,
                    WorkflowCostClass::InspectionNarrow,
                    WorkflowBudgetClass::InspectionBounded,
                    WorkflowFreshnessPolicy::ExactBasis,
                )),
                Some(WorthQueryWorkflowInspectionSemantics::post_merge_from_merge(lowered_merge)),
            ),
        ),
    ));

    let denied = match denied {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denial, got {other:?}"),
    };
    assert_eq!(
        denied.kind(),
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    );
}

fn runtime_lowered_merge() -> crate::workflow::LoweredMergeWorkflowDeclaration {
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
    .expect("merge declaration should admit");
    lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("runtime merge lowering should succeed")
}

fn preview_lowered_merge() -> crate::workflow::LoweredMergeWorkflowDeclaration {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("workflow-conflict-preview");
    let preview_binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("promotion preview binding should succeed");
    let foundation = admit_preview_workflow_foundation(&preview_binding)
        .expect("promotion workflow foundation should admit");
    let workflow_binding =
        bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
            .expect("preview workflow binding should succeed");
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
    lower_merge_workflow_declaration(
        &declaration,
        MergeLoweringInput::reconcile_into_target(
            BranchId("main".to_string()),
            BranchId("candidate".to_string()),
        ),
    )
    .expect("preview merge lowering should succeed")
}

fn runtime_lowered_writeback() -> crate::workflow::QueryWritebackDeclaration {
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
    .expect("writeback declaration should admit");
    lower_query_writeback_declaration(&declaration, WritebackLoweringInput::projected_state_diff())
        .expect("runtime writeback lowering should succeed")
}
