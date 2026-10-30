use super::authority::{BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner};
use super::evidence_reference::BridgeCausalEvidenceReference;
use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part, shape_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeCausalEvidenceBindingClass {
    RetainedBridgeRecord,
    ExternalAuthorityReference,
    RetainedNativeObservation,
    QueryObservationReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEvidenceBinding {
    owner: BridgeCausalEvidenceOwner,
    family: BridgeCausalEvidenceFamily,
    binding_class: BridgeCausalEvidenceBindingClass,
    reference_identity: BridgeIdentityEvidence,
    retained_record_identity: Option<BridgeIdentityEvidence>,
    binding_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEvidenceBinding {
    pub(super) fn query_observation_reporting(
        reference: &BridgeCausalEvidenceReference,
        binding_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self {
            owner: reference.owner(),
            family: reference.family(),
            binding_class: BridgeCausalEvidenceBindingClass::QueryObservationReport,
            reference_identity: reference.reference_evidence_identity().clone(),
            retained_record_identity: None,
            binding_identity,
        }
    }

    pub(super) fn retained(
        reference: &BridgeCausalEvidenceReference,
        retained_record_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self::new(
            reference,
            BridgeCausalEvidenceBindingClass::RetainedBridgeRecord,
            Some(retained_record_identity),
        )
    }

    pub(super) fn external(reference: &BridgeCausalEvidenceReference) -> Self {
        Self::new(
            reference,
            BridgeCausalEvidenceBindingClass::ExternalAuthorityReference,
            None,
        )
    }

    pub(super) fn native(reference: &BridgeCausalEvidenceReference) -> Self {
        Self::new(
            reference,
            BridgeCausalEvidenceBindingClass::RetainedNativeObservation,
            None,
        )
    }

    fn new(
        reference: &BridgeCausalEvidenceReference,
        binding_class: BridgeCausalEvidenceBindingClass,
        retained_record_identity: Option<BridgeIdentityEvidence>,
    ) -> Self {
        let external_retained_identity = external_retained_record_identity();
        let retained_identity = retained_record_identity
            .as_ref()
            .unwrap_or(&external_retained_identity);
        let mut parts = vec![
            shape_part(reference.owner().as_str()),
            shape_part(reference.family().as_str()),
            evidence_part(reference.reference_evidence_identity()),
            evidence_part(retained_identity),
        ];
        let domain = if binding_class == BridgeCausalEvidenceBindingClass::RetainedNativeObservation
        {
            parts.push(shape_part("retained_native_observation"));
            BridgeCausalEnvelopeDigestArtifact::NativeEvidenceBinding
        } else {
            BridgeCausalEnvelopeDigestArtifact::EvidenceBinding
        };
        let binding_identity = compose_bridge_causal_envelope_evidence_identity(domain, &parts);
        Self {
            owner: reference.owner(),
            family: reference.family(),
            binding_class,
            reference_identity: reference.reference_evidence_identity().clone(),
            retained_record_identity,
            binding_identity,
        }
    }

    pub fn owner(&self) -> BridgeCausalEvidenceOwner {
        self.owner
    }

    pub fn family(&self) -> BridgeCausalEvidenceFamily {
        self.family
    }

    pub fn binding_class(&self) -> BridgeCausalEvidenceBindingClass {
        self.binding_class
    }

    /// Descriptive original reference; this borrowed label supplies no authority.
    pub fn reference_for_reporting(&self) -> &str {
        self.reference_identity.terminal_projection_for_reporting()
    }

    pub fn reference_evidence_identity(&self) -> BridgeIdentityEvidence {
        self.reference_identity.clone()
    }

    pub fn retained_record_evidence_identity(&self) -> Option<BridgeIdentityEvidence> {
        self.retained_record_identity.clone()
    }

    pub fn retained_record_digest_for_reporting(&self) -> Option<&str> {
        self.retained_record_identity
            .as_ref()
            .map(BridgeIdentityEvidence::as_str)
    }

    pub fn binding_evidence_identity(&self) -> BridgeIdentityEvidence {
        self.binding_identity.clone()
    }

    pub fn binding_digest_for_reporting(&self) -> &str {
        self.binding_identity.as_str()
    }
}

fn external_retained_record_identity() -> BridgeIdentityEvidence {
    compose_bridge_causal_envelope_evidence_identity(
        BridgeCausalEnvelopeDigestArtifact::EvidenceBinding,
        &[shape_part("external-authority-reference")],
    )
}
