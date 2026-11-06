use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::super::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
};
use super::super::test_support::{admitted_plan_target_parts, declaration_target, ready, success};
use super::super::{
    materialize_query_workflow_declaration, WorthQueryWorkflowContributionAuthoring,
    WorthQueryWorkflowContributionPayload, WorthQueryWorkflowContributionPosture,
    WorthQueryWorkflowRuntimeBindingSemantics, WorthQueryWorkflowRuntimeSemantics,
};

#[test]
fn workflow_materializer_builds_preview_and_runtime_declarations(
) -> Result<(), IdentityDerivationDenial> {
    let preview = success(materialize_query_workflow_declaration(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
            "spatial.preview.only",
            "preview remains read-only",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:42",
            ),
        )?,
    )?)?);
    let confirmation = success(materialize_query_workflow_declaration(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection(
            "spatial.confirmation.required",
            "authoritative confirmation is required before writeback",
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:42")?,
        )?,
    )?)?);

    assert_eq!(
        preview.request().declaration_family(),
        &crate::workflow::WorkflowDeclarationFamily::ConflictInspectionNarrow
    );
    assert_eq!(
        preview.binding().basis_family(),
        &crate::workflow::WorkflowBasisFamily::PreviewFoundation
    );
    assert_eq!(
        preview.binding().preview_evaluation_class(),
        Some(&crate::workflow::WorkflowPreviewEvaluationClass::ReadOnly)
    );

    assert_eq!(
        confirmation.binding().basis_family(),
        &crate::workflow::WorkflowBasisFamily::RuntimePreflight
    );
    assert_eq!(
        confirmation
            .binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        Some(
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:42",)
                .evidence_identity()
        )
    );

    Ok(())
}

#[test]
fn workflow_materializer_accepts_admitted_plan_targets() -> Result<(), IdentityDerivationDenial> {
    let preview = success(materialize_query_workflow_declaration(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::promotion_eligible_mutation_lowering(
                "spatial.preview.lowering",
                "promotion-eligible preview can lower a bounded mutation workflow",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:77",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-workflow-preview",
                "request-preview",
                "eligibility-preview",
                "decision-preview",
            )?,
        )?,
    )?);
    let confirmation = success(materialize_query_workflow_declaration(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection(
                "spatial.confirmation.runtime",
                "authoritative confirmation requires runtime preflight context",
                crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:77")?,
            )?,
            admitted_plan_target_parts(
                "plan-workflow-runtime",
                "request-runtime",
                "eligibility-runtime",
                "decision-runtime",
            )?,
        )?,
    )?);

    assert_eq!(
        preview.binding().basis_family(),
        &crate::workflow::WorkflowBasisFamily::PreviewFoundation
    );
    assert_eq!(
        preview.binding().preview_evaluation_class(),
        Some(&crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible)
    );
    assert_eq!(
        preview.request().authority_target_family(),
        &crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation
    );
    assert_eq!(
        confirmation.binding().basis_family(),
        &crate::workflow::WorkflowBasisFamily::RuntimePreflight
    );
    assert_eq!(
        confirmation
            .binding()
            .runtime_snapshot_identity()
            .map(|identity| identity.evidence_identity()),
        Some(
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:77",)
                .evidence_identity()
        )
    );

    Ok(())
}

#[test]
fn discard_required_workflow_materializer_uses_promotion_eligible_preview_binding(
) -> Result<(), IdentityDerivationDenial> {
    let discard = success(materialize_query_workflow_declaration(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::discard_required_query_inspection(
            "spatial.preview.discard",
            "discard-required preview semantics should now carry promotable preview authority at the workflow seam",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                "preview-session:discard",
            ),
        )?,
    )?)?);

    assert_eq!(
        discard.binding().basis_family(),
        &crate::workflow::WorkflowBasisFamily::PreviewFoundation
    );
    assert_eq!(
        discard.binding().preview_evaluation_class(),
        Some(&crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible)
    );
    assert_eq!(
        discard.request().authority_target_family(),
        &crate::workflow::WorkflowAuthorityTargetFamily::QueryInspection
    );
    assert_eq!(
        discard.binding().preview_request_family(),
        Some(&crate::preview::PreviewWorkflowFoundationRequest::deferred_mutation_writeback())
    );

    Ok(())
}

#[test]
fn workflow_materialization_digest_changes_when_declaration_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_query_workflow_declaration(ready(
        super::super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-left"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::PreviewOnly,
                "spatial.preview.only",
                "preview remains read-only",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:42",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::ReadOnly,
                    ),
                    crate::workflow::WorkflowDeclarationFamily::ConflictInspectionNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::QueryInspection,
                    crate::workflow::WorkflowCostClass::InspectionNarrow,
                    crate::workflow::WorkflowBudgetClass::InspectionBounded,
                    crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
                )),
            )?,
        )?,
    )?)?);
    let right = success(materialize_query_workflow_declaration(ready(
        super::super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-right"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::PreviewOnly,
                "spatial.preview.only",
                "preview remains read-only",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:42",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::ReadOnly,
                    ),
                    crate::workflow::WorkflowDeclarationFamily::ConflictInspectionNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::QueryInspection,
                    crate::workflow::WorkflowCostClass::InspectionNarrow,
                    crate::workflow::WorkflowBudgetClass::InspectionBounded,
                    crate::workflow::WorkflowFreshnessPolicy::ExactBasis,
                )),
            )?,
        )?,
    )?)?);

    assert_ne!(
        left.report().declaration_digest(),
        right.report().declaration_digest()
    );
    assert_ne!(
        left.binding().binding_digest(),
        right.binding().binding_digest()
    );

    Ok(())
}

#[test]
fn workflow_materialization_digest_changes_when_admitted_plan_scope_changes(
) -> Result<(), IdentityDerivationDenial> {
    let left = success(materialize_query_workflow_declaration(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
                "spatial.preview.only",
                "preview remains read-only",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:42",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-workflow-left",
                "request-left",
                "eligibility-left",
                "decision-left",
            )?,
        )?,
    )?);
    let right = success(materialize_query_workflow_declaration(
        ready_workflow_plan(
            WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
                "spatial.preview.only",
                "preview remains read-only",
                crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                    "preview-session:42",
                ),
            )?,
            admitted_plan_target_parts(
                "plan-workflow-right",
                "request-right",
                "eligibility-right",
                "decision-right",
            )?,
        )?,
    )?);

    assert_ne!(
        left.report().declaration_digest(),
        right.report().declaration_digest()
    );
    assert_ne!(
        left.binding().binding_digest(),
        right.binding().binding_digest()
    );

    Ok(())
}

#[test]
fn workflow_materializer_denies_missing_or_inconsistent_runtime_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let missing = materialize_query_workflow_declaration(ready(
        super::super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow"),
            WorthQueryWorkflowContributionPayload::new(
                WorthQueryWorkflowContributionPosture::PreviewOnly,
                "spatial.preview.only",
                "preview remains read-only",
            )?,
        )?,
    )?)?;
    let inconsistent = materialize_query_workflow_declaration(ready(
        super::super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::PreviewOnly,
                "spatial.preview.only",
                "preview remains read-only",
                Some(WorthQueryWorkflowRuntimeSemantics::new(
                    WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                        crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                            "preview-session:42",
                        ),
                        crate::workflow::WorkflowPreviewEvaluationClass::PromotionEligible,
                    ),
                    crate::workflow::WorkflowDeclarationFamily::MutationLoweringNarrow,
                    crate::workflow::WorkflowAuthorityTargetFamily::RelationalMutation,
                    crate::workflow::WorkflowCostClass::MutationLoweringNarrow,
                    crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded,
                    crate::workflow::WorkflowFreshnessPolicy::AllowExplicitRebind,
                )),
            )?,
        )?,
    )?)?;

    assert!(matches!(
        missing,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::MissingCanonicalMaterializationSemantics
    ));
    assert!(matches!(
        inconsistent,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics
    ));

    Ok(())
}

fn ready_workflow(
    authoring: WorthQueryWorkflowContributionAuthoring,
) -> Result<
    super::super::WorthQueryMaterializationReadyWorkflowContribution<
        WorthQueryDeclarationBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(authoring.bind_to_declaration_target(
        declaration_target("intent-workflow"),
    ))?)
}

fn ready_workflow_plan(
    authoring: WorthQueryWorkflowContributionAuthoring,
    target: WorthQueryAdmittedPlanBoundContributionTarget,
) -> Result<
    super::super::WorthQueryMaterializationReadyWorkflowContribution<
        WorthQueryAdmittedPlanBoundContributionTarget,
    >,
    IdentityDerivationDenial,
> {
    Ok(ready(authoring.bind_to_admitted_plan_target(target))?)
}
