use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
    WorthQueryDomainCapabilityRebindRequired, WorthQueryDomainCapabilityStale,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTransitionOutcome, WorthQueryMaterializationReadyWorkflowContribution,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::{
    lower_merge_workflow_declaration, lower_mutation_intent_declaration,
    lower_query_writeback_declaration, LoweredMergeWorkflowDeclaration,
    LoweredMutationIntentDeclaration, QueryWritebackDeclaration, WorkflowLoweringError,
    WorkflowStalenessClass,
};

use super::materialize_query_workflow_declaration;
use super::semantics::{
    inconsistent_workflow_lowering_semantics_denial, missing_workflow_lowering_semantics_denial,
    workflow_lowering_semantics_match_runtime,
};
use super::WorthQueryWorkflowDeclarationMaterializationTarget;

#[cfg(test)]
pub fn materialize_lowered_mutation_intent_declaration<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<LoweredMutationIntentDeclaration>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
    let target_kind = contribution.payload().target().kind();
    let target_identity = contribution
        .payload()
        .target()
        .target_identity()
        .clone()
        .expect("positive fixture identity fits canonical digest bounds");
    let request_identity = contribution.payload().request_identity().clone();
    let lowering = match extract_workflow_lowering_semantics(
        "mutation workflow lowering",
        contribution.payload(),
    ) {
        Ok(lowering) => lowering.clone(),
        Err(denial) => return TransitionOutcome::Denied(denial),
    };
    let (authority_binding_identity, input) = match lowering.mutation_parts() {
        Some(parts) => parts,
        None => {
            return TransitionOutcome::Denied(inconsistent_workflow_lowering_semantics_denial(
                "mutation workflow lowering",
                contribution.payload().payload(),
                target_kind,
                request_identity,
            ))
        }
    };
    let declaration = match materialize_query_workflow_declaration(contribution)
        .expect("positive fixture identity fits canonical digest bounds")
    {
        TransitionOutcome::Success(declaration) => declaration,
        TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
        TransitionOutcome::Stale(stale) => return TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => {
            return TransitionOutcome::RebindRequired(rebind)
        }
        TransitionOutcome::Failed(failure) => return TransitionOutcome::Failed(failure),
        TransitionOutcome::Deferred(never) => match never {},
    };

    lower_mutation_intent_declaration(&declaration, authority_binding_identity, input.clone())
        .map_or_else(
            |error| {
                lowering_error_outcome(
                    "workflow-preview",
                    target_kind,
                    request_identity,
                    target_identity,
                    error,
                )
                .expect("positive fixture identity fits canonical digest bounds")
            },
            TransitionOutcome::Success,
        )
}

#[cfg(test)]
pub fn materialize_lowered_merge_workflow_declaration<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<LoweredMergeWorkflowDeclaration>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
    let target_kind = contribution.payload().target().kind();
    let target_identity = contribution
        .payload()
        .target()
        .target_identity()
        .clone()
        .expect("positive fixture identity fits canonical digest bounds");
    let request_identity = contribution.payload().request_identity().clone();
    let lowering = match extract_workflow_lowering_semantics(
        "merge workflow lowering",
        contribution.payload(),
    ) {
        Ok(lowering) => lowering.clone(),
        Err(denial) => return TransitionOutcome::Denied(denial),
    };
    let input = match lowering.merge_input() {
        Some(input) => input.clone(),
        None => {
            return TransitionOutcome::Denied(inconsistent_workflow_lowering_semantics_denial(
                "merge workflow lowering",
                contribution.payload().payload(),
                target_kind,
                request_identity,
            ))
        }
    };
    let declaration = match materialize_query_workflow_declaration(contribution)
        .expect("positive fixture identity fits canonical digest bounds")
    {
        TransitionOutcome::Success(declaration) => declaration,
        TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
        TransitionOutcome::Stale(stale) => return TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => {
            return TransitionOutcome::RebindRequired(rebind)
        }
        TransitionOutcome::Failed(failure) => return TransitionOutcome::Failed(failure),
        TransitionOutcome::Deferred(never) => match never {},
    };

    lower_merge_workflow_declaration(&declaration, input).map_or_else(
        |error| {
            lowering_error_outcome(
                "workflow-preview",
                target_kind,
                request_identity,
                target_identity,
                error,
            )
            .expect("positive fixture identity fits canonical digest bounds")
        },
        TransitionOutcome::Success,
    )
}

#[cfg(test)]
pub fn materialize_query_writeback_lowering<T>(
    contribution: WorthQueryMaterializationReadyWorkflowContribution<T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<QueryWritebackDeclaration>
where
    T: WorthQueryWorkflowDeclarationMaterializationTarget,
{
    let target_kind = contribution.payload().target().kind();
    let target_identity = contribution
        .payload()
        .target()
        .target_identity()
        .clone()
        .expect("positive fixture identity fits canonical digest bounds");
    let request_identity = contribution.payload().request_identity().clone();
    let lowering = match extract_workflow_lowering_semantics(
        "writeback workflow lowering",
        contribution.payload(),
    ) {
        Ok(lowering) => lowering.clone(),
        Err(denial) => return TransitionOutcome::Denied(denial),
    };
    let input = match lowering.writeback_input() {
        Some(input) => input.clone(),
        None => {
            return TransitionOutcome::Denied(inconsistent_workflow_lowering_semantics_denial(
                "writeback workflow lowering",
                contribution.payload().payload(),
                target_kind,
                request_identity,
            ))
        }
    };
    let declaration = match materialize_query_workflow_declaration(contribution)
        .expect("positive fixture identity fits canonical digest bounds")
    {
        TransitionOutcome::Success(declaration) => declaration,
        TransitionOutcome::Denied(denial) => return TransitionOutcome::Denied(denial),
        TransitionOutcome::Stale(stale) => return TransitionOutcome::Stale(stale),
        TransitionOutcome::RebindRequired(rebind) => {
            return TransitionOutcome::RebindRequired(rebind)
        }
        TransitionOutcome::Failed(failure) => return TransitionOutcome::Failed(failure),
        TransitionOutcome::Deferred(never) => match never {},
    };

    lower_query_writeback_declaration(&declaration, input).map_or_else(
        |error| {
            lowering_error_outcome(
                "workflow-preview",
                target_kind,
                request_identity,
                target_identity,
                error,
            )
            .expect("positive fixture identity fits canonical digest bounds")
        },
        TransitionOutcome::Success,
    )
}

#[cfg(test)]
fn extract_workflow_lowering_semantics<'a, T>(
    operation_label: &'static str,
    contribution: &'a crate::domain_capabilities::WorthQueryDomainCapabilityContribution<
        crate::domain_capabilities::WorthQueryWorkflowContributionPayload,
        T,
    >,
) -> Result<
    &'a crate::domain_capabilities::WorthQueryWorkflowLoweringSemantics,
    WorthQueryDomainCapabilityProgressionDenial,
>
where
    T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
{
    let payload = contribution.payload();
    let Some(runtime_semantics) = payload.runtime_semantics() else {
        return Err(missing_workflow_lowering_semantics_denial(
            operation_label,
            payload,
            contribution.target().kind(),
            contribution.request_identity().clone(),
        ));
    };
    let Some(lowering_semantics) = payload.lowering_semantics() else {
        return Err(missing_workflow_lowering_semantics_denial(
            operation_label,
            payload,
            contribution.target().kind(),
            contribution.request_identity().clone(),
        ));
    };
    if !workflow_lowering_semantics_match_runtime(runtime_semantics, lowering_semantics) {
        return Err(inconsistent_workflow_lowering_semantics_denial(
            operation_label,
            payload,
            contribution.target().kind(),
            contribution.request_identity().clone(),
        ));
    }
    Ok(lowering_semantics)
}

#[cfg(test)]
fn lowering_error_outcome<S>(
    category: &'static str,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
    target_identity: WorthQueryEvidenceIdentity,
    error: WorkflowLoweringError,
) -> Result<WorthQueryDomainCapabilityTransitionOutcome<S>, IdentityDerivationDenial> {
    let posture_identity = workflow_lowering_posture_identity(&target_identity, &error)?;
    Ok(match error.staleness_class() {
        WorkflowStalenessClass::StaleDenied => TransitionOutcome::Stale(
            WorthQueryDomainCapabilityStale::new(category, target_identity, posture_identity),
        ),
        WorkflowStalenessClass::ExplicitRebindRequired => {
            TransitionOutcome::RebindRequired(WorthQueryDomainCapabilityRebindRequired::new(
                category,
                target_identity,
                posture_identity,
            ))
        }
        WorkflowStalenessClass::ExactBasisPreserved
        | WorkflowStalenessClass::AuthorityValidationRequired => {
            TransitionOutcome::Denied(lowering_error_denial(
                category,
                request_identity,
                target_kind,
                &target_identity,
                error,
            ))
        }
    })
}

#[cfg(test)]
fn lowering_error_denial(
    category: &'static str,
    request_identity: WorthQueryEvidenceIdentity,
    target_kind: crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind,
    target_identity: &WorthQueryEvidenceIdentity,
    error: WorkflowLoweringError,
) -> WorthQueryDomainCapabilityProgressionDenial {
    WorthQueryDomainCapabilityProgressionDenial::new(
        WorthQueryDomainCapabilityProgressionDenialKind::UnsupportedCanonicalMaterializationPosture,
        category,
        target_kind,
        request_identity,
        format!(
            "workflow lowering denied at target `{}` with `{:?}` / `{:?}`: {}",
            target_identity.as_str(),
            error.failure_class(),
            error.staleness_class(),
            error.message()
        ),
    )
}

#[cfg(test)]
fn workflow_lowering_posture_identity(
    target_identity: &WorthQueryEvidenceIdentity,
    error: &WorkflowLoweringError,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_domain_capability_workflow_lowering_posture_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("target"), target_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("failure"),
                error.failure_class().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("staleness"),
                error.staleness_class().as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("message"), error.message())
            .seal()?,
    )
}
