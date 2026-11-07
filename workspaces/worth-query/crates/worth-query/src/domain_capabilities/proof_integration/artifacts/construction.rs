use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::{Artifact, ProofSet, ProofSetAuthorizedBy};

use super::super::proofs::{
    admitted_contribution_proof, eligible_contribution_proof,
    materialization_ready_contribution_proof, requested_contribution_proof,
    AdmittedContributionProof, DomainCapabilityContributionAuthority, EligibleContributionProof,
    MaterializationReadyContributionProof,
};
use super::allowed_bindings::AllowedContributionBinding;
use super::contribution::{DomainCapabilityBasis, WorthQueryDomainCapabilityContribution};
use super::phase_artifacts::WorthQueryRequestedDomainCapabilityContribution;
use crate::domain_capabilities::payloads::WorthQueryDomainCapabilityPayload;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;

pub(crate) fn create_requested_domain_capability_contribution<P, T>(
    target: T,
    payload: P,
) -> Result<WorthQueryRequestedDomainCapabilityContribution<P, T>, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
    (P, T): AllowedContributionBinding<P, T>,
{
    let binding_basis = target.binding_identity()?.as_str().to_string();
    Ok(WorthQueryRequestedDomainCapabilityContribution(
        remint_with_phase(
            WorthQueryDomainCapabilityContribution::new(target, payload)?,
            binding_basis,
            requested_contribution_proof(),
        ),
    ))
}

pub(crate) fn remint_with_phase<Phase, P, T, Proofs>(
    payload: WorthQueryDomainCapabilityContribution<P, T>,
    basis: String,
    proofs: Proofs,
) -> Artifact<Phase, WorthQueryDomainCapabilityContribution<P, T>, Proofs, DomainCapabilityBasis>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
    Proofs: ProofSet + ProofSetAuthorizedBy<DomainCapabilityContributionAuthority>,
{
    Artifact::with_proofs_and_current_basis(
        payload,
        proofs,
        basis,
        worth_proof::AuthorityWitness::from_authority_marker(DomainCapabilityContributionAuthority),
    )
}

pub(crate) fn eligible_proof() -> EligibleContributionProof {
    eligible_contribution_proof()
}

pub(crate) fn admitted_proof() -> AdmittedContributionProof {
    admitted_contribution_proof()
}

pub(crate) fn materialization_ready_proof() -> MaterializationReadyContributionProof {
    materialization_ready_contribution_proof()
}
