use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::Artifact;

use super::contribution::{
    contribution_basis_identity, DomainCapabilityBasis, WorthQueryDomainCapabilityContribution,
};
use super::phase_artifacts::{
    WorthQueryAdmittedDomainCapabilityContribution, WorthQueryEligibleDomainCapabilityContribution,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
    WorthQueryRequestedDomainCapabilityContribution,
};
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::domain_capabilities::payloads::WorthQueryDomainCapabilityPayload;
use crate::domain_capabilities::targets::WorthQueryDomainCapabilityTargetBinding;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

fn phase_identity<Phase, P, T, S>(
    phase: &'static str,
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
        domain_capability_scope_encoder("worth_query_domain_capability_phase_v1")
            .field_shape(WorthQueryEvidenceTag::new("phase"), phase)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                artifact.payload().request_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis"),
                &contribution_basis_identity(artifact)?,
            )
            .seal()?,
    )
}

impl<P, T> WorthQueryRequestedDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub fn requested_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(phase_identity("requested", &self.0)?)
    }

    pub fn requested_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.requested_identity()?.as_str().to_string())
    }
}

impl<P, T> WorthQueryEligibleDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub fn eligibility_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(phase_identity("eligible", &self.0)?)
    }

    pub fn eligibility_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.eligibility_identity()?.as_str().to_string())
    }
}

impl<P, T> WorthQueryAdmittedDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub fn admitted_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(phase_identity("admitted", &self.0)?)
    }

    pub fn admitted_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.admitted_identity()?.as_str().to_string())
    }
}

impl<P, T> WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    pub fn materialization_ready_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(phase_identity("materialization-ready", &self.0)?)
    }

    pub fn materialization_ready_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.materialization_ready_identity()?.as_str().to_string())
    }
}
