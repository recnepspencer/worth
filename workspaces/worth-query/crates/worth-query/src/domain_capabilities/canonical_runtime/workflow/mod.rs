use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_capabilities::payloads::WorthQueryWorkflowRuntimeBindingSemantics;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryDomainCapabilityTargetBinding,
};
use crate::domain_capabilities::{
    WorthQueryCanonicalWorkflowArtifact, WorthQueryDomainCapabilityTransitionOutcome,
    WorthQueryMaterializationReadyWorkflowContribution,
};
use crate::workflow::{
    admit_query_workflow_declaration, bind_workflow_context,
    scoped_runtime_preflight_workflow_binding_for_binding_identity, QueryWorkflowDeclaration,
    WorkflowBindingScopeField, WorkflowBindingSource, WorkflowDeclarationRequest,
};

use self::semantics::{
    inconsistent_workflow_runtime_semantics_denial, missing_workflow_runtime_semantics_denial,
    workflow_runtime_semantics_match_posture, workflow_source_label,
};

pub fn materialize_canonical_workflow_artifact<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
) -> WorthQueryCanonicalWorkflowArtifact<T>
where
    T: WorthQueryDomainCapabilityTargetBinding,
{
    super::artifacts::materialize_domain_capability_canonical_runtime_artifact(contribution)
}

pub fn materialize_query_workflow_declaration<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<QueryWorkflowDeclaration>,
    IdentityDerivationDenial,
>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
    let domain_contribution = contribution.payload();
    let payload = domain_contribution.payload();
    let target = domain_contribution.target();
    let Some(runtime_semantics) = payload.runtime_semantics() else {
        return Ok(TransitionOutcome::Denied(
            missing_workflow_runtime_semantics_denial(
                "workflow declaration materialization",
                payload,
                target.kind(),
                domain_contribution.request_identity().clone(),
            )?,
        ));
    };
    if !workflow_runtime_semantics_match_posture(payload.posture(), runtime_semantics) {
        return Ok(TransitionOutcome::Denied(
            inconsistent_workflow_runtime_semantics_denial(
                "workflow declaration materialization",
                payload,
                runtime_semantics,
                target.kind(),
                domain_contribution.request_identity().clone(),
            )?,
        ));
    }

    let source_label = workflow_source_label(target, payload);
    let binding = match runtime_semantics.binding() {
        WorthQueryWorkflowRuntimeBindingSemantics::RuntimePreflight {
            runtime_snapshot_identity,
        } => {
            let binding_scope =
                WorkflowBindingScopeField::Identity(&target.binding_identity());
            crate::workflow::synthetic_runtime_workflow_binding_scoped_for_snapshot_binding_identity(
                source_label.as_str(),
                &binding_scope,
                runtime_snapshot_identity.clone(),
            )
        }
        WorthQueryWorkflowRuntimeBindingSemantics::RuntimePreflightBundle { preflight } =>
            match scoped_runtime_preflight_workflow_binding_for_binding_identity(
                preflight,
                &target.binding_identity(),
            ) {
                Ok(binding) => binding,
                Err(error) => {
                    return Ok(TransitionOutcome::Denied(
                        WorthQueryDomainCapabilityProgressionDenial::new(
                            WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                            "workflow-preview",
                            domain_contribution.target().kind(),
                            domain_contribution.request_identity().clone(),
                            format!(
                                "workflow runtime-preflight binding admission denied with `{:?}`: {}",
                                error.failure_class(),
                                error.message()
                            ),
                        )?,
                    ))
                }
            },
        WorthQueryWorkflowRuntimeBindingSemantics::PreviewFoundation { .. } => {
            let foundation = match preview::admit_validated_preview_workflow_foundation(
                &contribution,
                runtime_semantics,
            ) {
                Ok(foundation) => foundation,
                Err(denial) => return Ok(TransitionOutcome::Denied(denial)),
            };
            match bind_workflow_context(WorkflowBindingSource::PreviewFoundation(&foundation)) {
                Ok(binding) => binding,
                Err(error) => {
                    return Ok(TransitionOutcome::Denied(
                        WorthQueryDomainCapabilityProgressionDenial::new(
                            WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
                            "workflow-preview",
                            domain_contribution.target().kind(),
                            domain_contribution.request_identity().clone(),
                            format!(
                                "workflow preview binding admission denied with `{:?}`: {}",
                                error.failure_class(),
                                error.message()
                            ),
                        )?,
                    ))
                }
            }
        }
    };
    let request = WorkflowDeclarationRequest::new(
        runtime_semantics.declaration_family().clone(),
        runtime_semantics.authority_target_family().clone(),
        runtime_semantics.cost_class().clone(),
        runtime_semantics.budget_class().clone(),
        runtime_semantics.freshness_policy().clone(),
    );

    match admit_query_workflow_declaration(&binding, request) {
        Ok(declaration) => TransitionOutcome::Success(declaration),
        Err(error) => TransitionOutcome::Denied(WorthQueryDomainCapabilityProgressionDenial::new(
            WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
            "workflow-preview",
            domain_contribution.target().kind(),
            domain_contribution.request_identity().clone(),
            format!(
                "workflow declaration materialization denied with `{:?}`: {}",
                error.failure_class(),
                error.message()
            ),
        )?),
    }

    Ok(())
}

pub trait WorthQueryWorkflowDeclarationMaterializationTarget:
    WorthQueryDomainCapabilityTargetBinding
    + preview::WorthQueryWorkflowPreviewMaterializationTarget
    + private::Sealed
{
}

impl WorthQueryWorkflowDeclarationMaterializationTarget
    for WorthQueryDeclarationBoundContributionTarget
{
}

impl WorthQueryWorkflowDeclarationMaterializationTarget
    for WorthQueryAdmittedPlanBoundContributionTarget
{
}

impl<T> WorthQueryWorkflowDeclarationMaterializationTarget
    for crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
}

mod private {
    pub trait Sealed {}
}

impl private::Sealed for WorthQueryDeclarationBoundContributionTarget {}
impl private::Sealed for WorthQueryAdmittedPlanBoundContributionTarget {}
impl<T> private::Sealed
    for crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
}

#[cfg(test)]
mod inspection;
#[cfg(test)]
mod lowering;
mod preview;
mod preview_identity;
mod semantics;

#[cfg(test)]
pub use inspection::*;
#[cfg(test)]
pub use lowering::*;
pub use preview::*;
