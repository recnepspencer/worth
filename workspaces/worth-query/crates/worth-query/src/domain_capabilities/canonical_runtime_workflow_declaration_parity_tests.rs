use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::targets::WorthQueryDeclarationBoundContributionTarget;
use super::test_support::{declaration_target, ready, success};
use super::{
    materialize_admitted_preview_workflow_foundation, materialize_query_workflow_declaration,
    WorthQueryWorkflowContributionAuthoring,
};
use crate::workflow::{bind_workflow_context, WorkflowBindingSource};

#[test]
fn workflow_materializer_reuses_admitted_preview_foundation_identity(
) -> Result<(), IdentityDerivationDenial> {
    let declaration = success(materialize_query_workflow_declaration(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
            "spatial.preview.identity",
            "workflow declarations should bind through the same admitted preview foundation identity",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name("preview-session:identity"),
        )?,
    )?)?);
    let foundation = success(materialize_admitted_preview_workflow_foundation(ready_workflow(
        WorthQueryWorkflowContributionAuthoring::preview_only_query_inspection(
            "spatial.preview.identity",
            "workflow declarations should bind through the same admitted preview foundation identity",
            crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name("preview-session:identity"),
        )?,
    )?));

    let expected_binding =
        bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation))
            .expect("preview foundation should bind");

    assert_eq!(
        declaration.binding().source_for_reporting(),
        expected_binding.source_for_reporting()
    );
    assert_eq!(
        declaration.binding().query_for_reporting(),
        expected_binding.query_for_reporting()
    );
    assert_eq!(
        declaration.binding().basis_for_reporting(),
        expected_binding.basis_for_reporting()
    );
    assert_eq!(
        declaration.binding().preview_request_family(),
        Some(foundation.request_family())
    );

    Ok(())
}

#[test]
fn workflow_materializer_denies_runtime_only_postures_at_preview_foundation_boundary(
) -> Result<(), IdentityDerivationDenial> {
    let denied = materialize_query_workflow_declaration(ready(super::proof_integration::create_requested_domain_capability_contribution(
        declaration_target("intent-workflow-discard-read-only-denied"),
        super::WorthQueryWorkflowContributionPayload::with_runtime_semantics(
            super::WorthQueryWorkflowContributionPosture::DiscardRequired,
            "spatial.preview.discard.read_only_denied",
            "workflow declaration materialization must deny the same dishonest read-only discard-required preview semantics as preview foundation admission",
            Some(super::WorthQueryWorkflowRuntimeSemantics::new(
                super::WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    crate::facade::runtime::BridgePreviewSessionIdentity::from_stable_name(
                        "preview-session:denied",
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
    )?)?)?;

    assert!(matches!(
        denied,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics
    ));

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
    ready(
        authoring
            .bind_to_declaration_target(declaration_target("intent-workflow"))
            .expect("fixture digest must fit"),
    )
}
