use super::authority::{BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner};
use super::counters::BridgeCausalEnvelopeCounters;
use super::denial::{BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind};
use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part, shape_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEvidenceReferenceIdentity {
    family: BridgeCausalEvidenceFamily,
    identity: BridgeIdentityEvidence,
}

impl BridgeCausalEvidenceReferenceIdentity {
    fn for_owner(
        owner: BridgeCausalEvidenceOwner,
        family: BridgeCausalEvidenceFamily,
        identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        if identity.is_empty() {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::EmptyEvidenceReference,
                family,
                owner,
                family.expected_owner(),
                identity.clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
        if owner != family.expected_owner() {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::EvidenceOwnerMismatch,
                family,
                owner,
                family.expected_owner(),
                identity.clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
        Ok(Self { family, identity })
    }

    pub fn query_observation(
        identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::for_owner(
            BridgeCausalEvidenceOwner::Query,
            BridgeCausalEvidenceFamily::QueryObservation,
            identity,
        )
    }

    pub fn runtime_bridge(
        family: BridgeCausalEvidenceFamily,
        identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::for_owner(BridgeCausalEvidenceOwner::RuntimeBridge, family, identity)
    }

    pub fn relational_authority(
        identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::for_owner(
            BridgeCausalEvidenceOwner::Relational,
            BridgeCausalEvidenceFamily::RelationalAuthority,
            identity,
        )
    }

    pub fn signal(
        family: BridgeCausalEvidenceFamily,
        identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::for_owner(BridgeCausalEvidenceOwner::Signal, family, identity)
    }

    pub fn family(&self) -> BridgeCausalEvidenceFamily {
        self.family
    }

    pub fn evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEvidenceReference {
    owner: BridgeCausalEvidenceOwner,
    family: BridgeCausalEvidenceFamily,
    reference_digest_identity: BridgeIdentityEvidence,
    reference_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEvidenceReference {
    pub(super) fn query_observation_reporting(
        observation: &BridgeIdentityEvidence,
        reference_digest_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self {
            owner: BridgeCausalEvidenceOwner::Query,
            family: BridgeCausalEvidenceFamily::QueryObservation,
            reference_digest_identity,
            reference_identity: observation.clone(),
        }
    }

    pub fn new(
        owner: BridgeCausalEvidenceOwner,
        family: BridgeCausalEvidenceFamily,
        reference_identity: BridgeCausalEvidenceReferenceIdentity,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        if reference_identity.family() != family {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::EvidenceReferenceFamilyMismatch,
                family,
                owner,
                reference_identity.family().expected_owner(),
                reference_identity.evidence_identity().clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
        if owner != family.expected_owner() {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::EvidenceOwnerMismatch,
                family,
                owner,
                family.expected_owner(),
                reference_identity.evidence_identity().clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
        let reference_identity = reference_identity.identity.clone();
        let reference_digest_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::EvidenceReference,
            &[
                shape_part(owner.as_str()),
                shape_part(family.as_str()),
                evidence_part(&reference_identity),
            ],
        );
        Ok(Self {
            owner,
            family,
            reference_digest_identity,
            reference_identity,
        })
    }

    pub fn owner(&self) -> BridgeCausalEvidenceOwner {
        self.owner
    }

    pub fn family(&self) -> BridgeCausalEvidenceFamily {
        self.family
    }

    pub fn reference_digest_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.reference_digest_identity
    }

    pub fn reference_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.reference_identity
    }
}
