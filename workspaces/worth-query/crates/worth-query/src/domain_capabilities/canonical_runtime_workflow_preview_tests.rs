use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod posture_denials;

use worth_proof::TransitionOutcome;

use super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryDomainCapabilityTargetBinding,
};
use super::test_support::{admitted_plan_target_parts, declaration_target, ready, success};
use super::{
    materialize_admitted_preview_workflow_foundation, materialize_query_preview_workflow_artifact,
    WorthQueryWorkflowContributionAuthoring, WorthQueryWorkflowContributionPayload,
    WorthQueryWorkflowContributionPosture,
};

#[test]
fn preview_digest_lowering_carries_foundational_identity_without_rehashing_text() {
    let source = include_str!("canonical_runtime/workflow/preview_identity.rs");

    assert!(source.contains("from_canonical_digest(identity.canonical_digest())"));
    assert!(!source.contains("::from_parts"));
    assert!(!source.contains("hash_parts"));
    assert!(!source.contains("Sha256"));
}

#[test]
fn preview_workflow_artifact_materializer_builds_preview_artifacts(
) -> Result<(), IdentityDerivationDenial> {
    let read_only = success(materialize_query_preview_workflow_artifact(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
            "spatial.preview.only",
            "preview remains read-only",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:42",
            ),
        )?,
    )?));
    let promotion = success(materialize_query_preview_workflow_artifact(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
                "spatial.preview.lowering",
                "promotion-eligible preview can lower bounded mutation workflow",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:77",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-preview-promotion",
                "request-preview",
                "eligibility-preview",
                "decision-preview",
            )?,
        )?,
    ));
    let discard = success(materialize_query_preview_workflow_artifact(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::discard_required_query_inspection(
            "spatial.preview.discard",
            "preview must discard rather than promote",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:99",
            ),
        )?,
    )?));

    assert_eq!(
        read_only.request_family(),
        &crate::preview::PreviewWorkflowFoundationRequest::compare_basis_pair()
    );
    assert_eq!(
        read_only
            .preview_session_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        "preview-session:42"
    );
    assert_eq!(
        read_only.evaluation_class(),
        &crate::preview::PreviewEvaluationClass::read_only()
    );
    assert_eq!(
        promotion
            .preview_session_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        "preview-session:77"
    );
    assert_eq!(
        promotion.request_family(),
        &crate::preview::PreviewWorkflowFoundationRequest::compare_basis_pair()
    );
    assert_eq!(
        promotion.evaluation_class(),
        &crate::preview::PreviewEvaluationClass::promotion_eligible()
    );
    assert_eq!(
        promotion.binding_for_reporting(),
        admitted_plan_target_parts(
            "plan-preview-promotion",
            "request-preview",
            "eligibility-preview",
            "decision-preview",
        )?
        .binding_identity()?
        .as_str()
    );
    assert_eq!(
        discard.request_family(),
        &crate::preview::PreviewWorkflowFoundationRequest::deferred_mutation_writeback()
    );

    Ok(())
}

#[test]
fn preview_workflow_artifact_digest_changes_when_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_query_preview_workflow_artifact(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
                "spatial.preview.only",
                "preview remains read-only",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:42",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-preview-left",
                "request-left",
                "eligibility-left",
                "decision-left",
            )?,
        )?,
    ));
    let right = success(materialize_query_preview_workflow_artifact(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
                "spatial.preview.only",
                "preview remains read-only",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:42",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-preview-right",
                "request-right",
                "eligibility-right",
                "decision-right",
            )?,
        )?,
    ));

    assert_ne!(
        left.artifact_for_reporting(),
        right.artifact_for_reporting()
    );
    assert_ne!(
        left.declaration_digest_for_reporting(),
        right.declaration_digest_for_reporting()
    );
    assert_ne!(
        left.canonical_query_digest().as_str(),
        right.canonical_query_digest().as_str()
    );

    Ok(())
}

#[test]
fn preview_workflow_artifact_separates_request_family_in_identity_basis(
) -> Result<(), IdentityDerivationDenial> {
    let promotion = success(materialize_query_preview_workflow_artifact(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-preview-request-family"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::PromotionEligible,
                "spatial.preview.identity.same",
                "request family must participate in preview declaration identity",
                Some(super::WorthQueryWorkflowRuntimeSemantics::new(
                    super::WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:identity",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                    ),
                    crate::workflow::WorkflowDeclarationFamily::MutationLoweringNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation,
                    crate::workflow::WorkflowCostClass::MutationLoweringNarrow,
                    crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                    crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
                )),
            )?,
        )?,
    )?));
    let discard = success(materialize_query_preview_workflow_artifact(ready(
        super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-preview-request-family"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::DiscardRequired,
                "spatial.preview.identity.same",
                "request family must participate in preview declaration identity",
                Some(super::WorthQueryWorkflowRuntimeSemantics::new(
                    super::WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:identity",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                    ),
                    crate::workflow::WorkflowDeclarationFamily::MutationLoweringNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation,
                    crate::workflow::WorkflowCostClass::MutationLoweringNarrow,
                    crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                    crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
                )),
            )?,
        )?,
    )?));

    assert_ne!(
        promotion
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting(),
        discard
            .declaration_identity()
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
    );
    assert_ne!(
        promotion.declaration_digest_for_reporting(),
        discard.declaration_digest_for_reporting()
    );
    assert_ne!(
        promotion.canonical_query_digest().as_str(),
        discard.canonical_query_digest().as_str()
    );

    Ok(())
}

#[test]
fn admitted_preview_workflow_foundation_materializer_builds_real_foundations(
) -> Result<(), IdentityDerivationDenial> {
    let read_only = success(materialize_admitted_preview_workflow_foundation(
        ready_workflow(
            WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
                "spatial.preview.only",
                "preview remains read-only",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:42",
                ),
            )?,
        )?,
    ));
    let promotion = success(materialize_admitted_preview_workflow_foundation(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
                "spatial.preview.lowering",
                "promotion-eligible preview can lower bounded mutation workflow",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:77",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-preview-promotion",
                "request-preview",
                "eligibility-preview",
                "decision-preview",
            )?,
        )?,
    ));

    assert_eq!(
        read_only.request_family(),
        &crate::preview::PreviewWorkflowFoundationRequest::compare_basis_pair()
    );
    assert_eq!(
        read_only.evaluation_class(),
        &crate::preview::PreviewEvaluationClass::read_only()
    );
    assert_eq!(
        promotion.evaluation_class(),
        &crate::preview::PreviewEvaluationClass::promotion_eligible()
    );
    assert_eq!(
        read_only
            .counters()
            .preview_workflow_foundation_admission_count(),
        1
    );
    assert_eq!(
        promotion
            .counters()
            .preview_workflow_foundation_artifact_lookup_count(),
        1
    );

    Ok(())
}

#[test]
fn admitted_preview_workflow_foundation_admits_discard_required_requests(
) -> Result<(), IdentityDerivationDenial> {
    let admitted = success(materialize_admitted_preview_workflow_foundation(
        ready_workflow(
            WorthQueryWorkflowContributionAuthoring::discard_required_query_inspection(
                "spatial.preview.discard",
                "preview must discard rather than promote",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:66",
                ),
            )?,
        )?,
    ));

    assert_eq!(
        admitted.request_family(),
        &crate::preview::PreviewWorkflowFoundationRequest::deferred_mutation_writeback()
    );
    assert_eq!(
        admitted.evaluation_class(),
        &crate::preview::PreviewEvaluationClass::promotion_eligible()
    );
    assert_eq!(
        admitted
            .counters()
            .preview_workflow_foundation_admission_count(),
        1
    );
    assert_eq!(
        admitted
            .counters()
            .preview_workflow_foundation_denial_count(),
        0
    );

    Ok(())
}

fn ready_workflow(
    authoring: WorthQueryWorkflowContributionAuthoring,
) -> Result<
    super::WorthQueryMaterializationReadyWorkflowContribution<
        WorthQueryDeclarationBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(
        authoring
            .bind_to_declaration_target(declaration_target("intent-workflow"))
            .expect("fixture digest must fit"),
    )?)
}

fn ready_workflow_plan(
    authoring: WorthQueryWorkflowContributionAuthoring,
    target: WorthQueryAdmittedPlanBoundContributionTarget,
) -> Result<
    super::WorthQueryMaterializationReadyWorkflowContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(
        authoring
            .bind_to_admitted_plan_target(target)
            .expect("fixture digest must fit"),
    )?)
}
