use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::{Artifact, AssumptionBasis, CurrentValidity, FreshnessScopedBasis};

use super::super::phases::{
    ContributionAdmittedPhase, ContributionEligiblePhase, ContributionMaterializationReadyPhase,
    ContributionRequestedPhase,
};
use super::super::proofs::{
    AdmittedContributionProof, EligibleContributionProof, MaterializationReadyContributionProof,
    RequestedContributionProof,
};
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::domain_capabilities::payloads::WorthQueryDomainCapabilityPayload;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(super) type DomainCapabilityBasis =
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<String>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    target: T,
    payload: P,
    request_identity: WorthQueryEvidenceIdentity,
}

impl<P, T> WorthQueryDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub fn category(&self) -> crate::domain_capabilities::WorthQueryDomainCapabilityCategory {
        self.payload.category()
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    pub fn payload(&self) -> &P {
        &self.payload
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn request_digest(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn installed_authority(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryInstalledDomainAuthority> {
        self.target.installed_authority()
    }

    pub(super) fn new(target: T, payload: P) -> Result<Self, IdentityDerivationDenial> {
        let request_identity = compose_domain_capability_request_identity(&target, &payload)?;
        Ok(Self {
            target,
            payload,
            request_identity,
        })
    }
}

fn compose_domain_capability_request_identity<P, T>(
    target: &T,
    payload: &P,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let binding_identity = target.binding_identity()?;
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_request_v2")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            payload.category().as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("payload"),
            payload.payload_identity(),
        );
    if let Some(authority) = target.installed_authority() {
        identity = identity
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("installed_authority"),
                authority.authority_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("installed_world"),
                authority.world_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("installed_package"),
                authority.package_identity().evidence_identity(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("installed_generation"),
                authority.installation_generation().ordinal() as usize,
            );
    }
    Ok(identity.seal()?)
}

pub(crate) fn contribution_basis_identity<Phase, P, T, S>(
    artifact: &Artifact<
        Phase,
        WorthQueryDomainCapabilityContribution<P, T>,
        S,
        DomainCapabilityBasis,
    >,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_basis_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("assumption"),
                contribution_basis(artifact).as_str(),
            )
            .seal()?,
    )
}

pub(crate) fn contribution_basis<Phase, P, T, S>(
    artifact: &Artifact<
        Phase,
        WorthQueryDomainCapabilityContribution<P, T>,
        S,
        DomainCapabilityBasis,
    >,
) -> String
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    artifact.basis().basis().value().clone()
}

pub(super) type RequestedContributionArtifact<P, T> = Artifact<
    ContributionRequestedPhase,
    WorthQueryDomainCapabilityContribution<P, T>,
    RequestedContributionProof,
    DomainCapabilityBasis,
>;
pub(super) type EligibleContributionArtifact<P, T> = Artifact<
    ContributionEligiblePhase,
    WorthQueryDomainCapabilityContribution<P, T>,
    EligibleContributionProof,
    DomainCapabilityBasis,
>;
pub(super) type AdmittedContributionArtifact<P, T> = Artifact<
    ContributionAdmittedPhase,
    WorthQueryDomainCapabilityContribution<P, T>,
    AdmittedContributionProof,
    DomainCapabilityBasis,
>;
pub(super) type MaterializationReadyContributionArtifact<P, T> = Artifact<
    ContributionMaterializationReadyPhase,
    WorthQueryDomainCapabilityContribution<P, T>,
    MaterializationReadyContributionProof,
    DomainCapabilityBasis,
>;
