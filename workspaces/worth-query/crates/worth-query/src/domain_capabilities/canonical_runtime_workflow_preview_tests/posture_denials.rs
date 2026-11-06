use super::*;

#[test]
fn preview_workflow_artifact_materializer_denies_runtime_only_workflow_postures(
) -> Result<(), IdentityDerivationDenial> {
    let denied = materialize_query_preview_workflow_artifact(ready_workflow_plan(
        WorthQueryWorkflowContributionAuthoring::confirmation_required_query_inspection(
            "spatial.confirmation.runtime",
            "authoritative confirmation requires runtime preflight context",
            crate::memory_workspace::admit_external_snapshot_label("runtime-snapshot:77"),
        )?,
        admitted_plan_target_parts(
            "plan-preview-runtime",
            "request-runtime",
            "eligibility-runtime",
            "decision-runtime",
        )?,
    ));

    assert!(matches!(
        denied,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture
    ));

    Ok(())
}

#[test]
fn preview_workflow_materializer_denies_read_only_discard_required_runtime_semantics(
) -> Result<(), IdentityDerivationDenial> {
    let denied = materialize_admitted_preview_workflow_foundation(ready(
        super::super::proof_integration::create_requested_domain_capability_contribution(
            declaration_target("intent-workflow-discard-read-only-denied"),
            WorthQueryWorkflowContributionPayload::with_runtime_semantics(
                WorthQueryWorkflowContributionPosture::DiscardRequired,
                "spatial.preview.discard.read_only_denied",
                "discard-required preview semantics must not pretend read-only foundations can carry deferred writeback authority",
                Some(super::super::WorthQueryWorkflowRuntimeSemantics::new(
                    super::super::WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
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
        ),
    ));

    assert!(matches!(
        denied,
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == super::super::WorthQueryDomainCapabilityProgressionDenialKind::InconsistentCanonicalMaterializationSemantics
    ));

    Ok(())
}
