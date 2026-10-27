use super::authority::{BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner};
use super::counters::BridgeCausalEnvelopeCounters;
use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part, shape_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeCausalEnvelopeDenialKind {
    EmptyAssemblyRequestDigest,
    EmptyEvidenceReference,
    DuplicateEvidenceReference,
    EvidenceOwnerMismatch,
    EvidenceReferenceFamilyMismatch,
    MissingEvidenceReference,
    MissingQueryObservationAnchor,
    QueryObservationAnchorOverclaim,
    MissingRequiredBridgeRouteEvidence,
    MissingRetainedBridgeRecord,
    NativeObservationRequired,
    ForeignNativeObservation,
    QueryObservationAssociationMismatch,
}

impl BridgeCausalEnvelopeDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmptyAssemblyRequestDigest => "empty_assembly_request_digest",
            Self::EmptyEvidenceReference => "empty_evidence_reference",
            Self::DuplicateEvidenceReference => "duplicate_evidence_reference",
            Self::EvidenceOwnerMismatch => "evidence_owner_mismatch",
            Self::EvidenceReferenceFamilyMismatch => "evidence_reference_family_mismatch",
            Self::MissingEvidenceReference => "missing_evidence_reference",
            Self::MissingQueryObservationAnchor => "missing_query_observation_anchor",
            Self::QueryObservationAnchorOverclaim => "query_observation_anchor_overclaim",
            Self::MissingRequiredBridgeRouteEvidence => "missing_required_bridge_route_evidence",
            Self::MissingRetainedBridgeRecord => "missing_retained_bridge_record",
            Self::NativeObservationRequired => "native_observation_required",
            Self::ForeignNativeObservation => "foreign_native_observation",
            Self::QueryObservationAssociationMismatch => "query_observation_association_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEnvelopeDenial {
    kind: BridgeCausalEnvelopeDenialKind,
    family: BridgeCausalEvidenceFamily,
    supplied_owner: BridgeCausalEvidenceOwner,
    expected_owner: BridgeCausalEvidenceOwner,
    reference_identity: BridgeIdentityEvidence,
    counters: BridgeCausalEnvelopeCounters,
    failure_digest: BridgeIdentityEvidence,
}

impl BridgeCausalEnvelopeDenial {
    pub(crate) fn new(
        kind: BridgeCausalEnvelopeDenialKind,
        family: BridgeCausalEvidenceFamily,
        supplied_owner: BridgeCausalEvidenceOwner,
        expected_owner: BridgeCausalEvidenceOwner,
        reference_identity: BridgeIdentityEvidence,
        counters: BridgeCausalEnvelopeCounters,
    ) -> Self {
        let kind_text = format!("{kind:?}");
        let failure_digest = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::Denial,
            &[
                shape_part(&kind_text),
                shape_part(family.as_str()),
                shape_part(supplied_owner.as_str()),
                shape_part(expected_owner.as_str()),
                evidence_part(&reference_identity),
                evidence_part(counters.counter_evidence_identity()),
            ],
        );
        Self {
            kind,
            family,
            supplied_owner,
            expected_owner,
            reference_identity,
            counters,
            failure_digest,
        }
    }

    pub fn kind(&self) -> BridgeCausalEnvelopeDenialKind {
        self.kind
    }

    pub fn family(&self) -> BridgeCausalEvidenceFamily {
        self.family
    }

    pub fn supplied_owner(&self) -> BridgeCausalEvidenceOwner {
        self.supplied_owner
    }

    pub fn expected_owner(&self) -> BridgeCausalEvidenceOwner {
        self.expected_owner
    }

    pub fn reference_identity_for_reporting(&self) -> &str {
        self.reference_identity.as_str()
    }

    pub fn reference_evidence_identity(&self) -> BridgeIdentityEvidence {
        self.reference_identity.clone()
    }

    pub fn counters(&self) -> &BridgeCausalEnvelopeCounters {
        &self.counters
    }

    pub fn failure_for_reporting(&self) -> &str {
        self.failure_digest.as_str()
    }

    pub fn failure_evidence_identity(&self) -> BridgeIdentityEvidence {
        self.failure_digest.clone()
    }
}
