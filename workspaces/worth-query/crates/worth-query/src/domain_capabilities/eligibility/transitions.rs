use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use crate::domain_capabilities::denials::{
    WorthQueryDomainCapabilityProgressionDenial, WorthQueryDomainCapabilityProgressionDenialKind,
    WorthQueryDomainCapabilityRebindRequired, WorthQueryDomainCapabilityStale,
    WorthQueryDomainCapabilityTransitionOutcome,
};
use crate::domain_capabilities::payloads::WorthQueryDomainCapabilityPayload;
use crate::domain_capabilities::proof_integration::{
    admitted_proof, contribution_basis, eligible_proof, materialization_ready_proof,
    remint_with_phase, WorthQueryAdmittedDomainCapabilityContribution,
    WorthQueryDomainCapabilityContribution, WorthQueryEligibleDomainCapabilityContribution,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
    WorthQueryRequestedDomainCapabilityContribution,
};
use crate::domain_capabilities::targets::{
    WorthQueryDomainCapabilityTargetBinding, WorthQueryDomainCapabilityTargetKind,
};

pub fn evaluate_requested_domain_capability_contribution<P, T>(
    requested: WorthQueryRequestedDomainCapabilityContribution<P, T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<WorthQueryEligibleDomainCapabilityContribution<P, T>>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    if let Some(denial) = stale_installation_generation_denial(requested.payload()) {
        return TransitionOutcome::Denied(denial);
    }
    if requested
        .payload()
        .installed_authority()
        .is_some_and(|authority| {
            !authority.permits_domain_capability_category(requested.payload().category())
        })
    {
        return TransitionOutcome::Denied(denial_for(
            requested.payload(),
            WorthQueryDomainCapabilityProgressionDenialKind::ContributionCategoryNotInstalled,
            "the installed domain package does not admit this contribution category",
        ));
    }
    if requested
        .payload()
        .payload()
        .semantic_code()
        .trim()
        .is_empty()
    {
        return TransitionOutcome::Denied(denial_for(
            requested.payload(),
            WorthQueryDomainCapabilityProgressionDenialKind::EmptySemanticCode,
            "domain capability contributions require a non-empty semantic code",
        ));
    }
    if requested.payload().payload().detail().trim().is_empty() {
        return TransitionOutcome::Denied(denial_for(
            requested.payload(),
            WorthQueryDomainCapabilityProgressionDenialKind::EmptyDetail,
            "domain capability contributions require non-empty detail text",
        ));
    }
    let basis = contribution_basis(&requested.0);
    let payload = requested.into_inner().into_parts().into_parts().0;
    TransitionOutcome::Success(WorthQueryEligibleDomainCapabilityContribution(
        remint_with_phase(payload, basis, eligible_proof()),
    ))
}

pub fn admit_eligible_domain_capability_contribution<P, T>(
    eligible: WorthQueryEligibleDomainCapabilityContribution<P, T>,
) -> WorthQueryDomainCapabilityTransitionOutcome<WorthQueryAdmittedDomainCapabilityContribution<P, T>>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    if let Some(denial) = stale_installation_generation_denial(eligible.payload()) {
        return TransitionOutcome::Denied(denial);
    }
    let basis = contribution_basis(&eligible.0);
    let payload = eligible.into_inner().into_parts().into_parts().0;
    TransitionOutcome::Success(WorthQueryAdmittedDomainCapabilityContribution(
        remint_with_phase(payload, basis, admitted_proof()),
    ))
}

pub fn prepare_admitted_domain_capability_contribution_for_materialization<P, T>(
    admitted: WorthQueryAdmittedDomainCapabilityContribution<P, T>,
    current_target: T,
) -> Result<
    WorthQueryDomainCapabilityTransitionOutcome<
        WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
    >,
    IdentityDerivationDenial,
>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    if let Some(denial) = stale_installation_generation_denial(admitted.payload()) {
        return Ok(TransitionOutcome::Denied(denial));
    }
    let basis = contribution_basis(&admitted.0);
    let payload = admitted.into_inner().into_parts().into_parts().0;
    let category = payload.category().as_str();
    let bound_target = payload.target();
    if bound_target.binding_identity() != current_target.binding_identity() {
        return Ok(match current_target.kind() {
            WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope => {
                TransitionOutcome::Stale(WorthQueryDomainCapabilityStale::new(
                    category,
                    bound_target.target_identity()?,
                    current_target.target_identity()?,
                ))
            }
            _ => TransitionOutcome::RebindRequired(WorthQueryDomainCapabilityRebindRequired::new(
                category,
                bound_target.target_identity()?,
                current_target.target_identity()?,
            )),
        });
    }

    Ok(TransitionOutcome::Success(
        WorthQueryMaterializationReadyDomainCapabilityContribution(remint_with_phase(
            payload,
            basis,
            materialization_ready_proof(),
        )),
    ))
}

fn stale_installation_generation_denial<P, T>(
    contribution: &WorthQueryDomainCapabilityContribution<P, T>,
) -> Option<WorthQueryDomainCapabilityProgressionDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    contribution
        .installed_authority()
        .is_some_and(|authority| !authority.is_current_installation_generation())
        .then(|| {
            denial_for(
                contribution,
                WorthQueryDomainCapabilityProgressionDenialKind::StaleInstallationGeneration,
                "the installed domain authority belongs to a stale installation generation",
            )
        })
}

fn denial_for<P, T>(
    contribution: &WorthQueryDomainCapabilityContribution<P, T>,
    kind: WorthQueryDomainCapabilityProgressionDenialKind,
    message: &str,
) -> WorthQueryDomainCapabilityProgressionDenial
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    WorthQueryDomainCapabilityProgressionDenial::new(
        kind,
        contribution.category().as_str(),
        contribution.target().kind(),
        contribution.request_identity().clone(),
        message,
    )
}
