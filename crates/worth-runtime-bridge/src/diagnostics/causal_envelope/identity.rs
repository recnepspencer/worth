use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEnvelopeIdentity {
    request_identity: BridgeIdentityEvidence,
    causal_observation_anchor_identity: BridgeIdentityEvidence,
    evidence_binding_identity: BridgeIdentityEvidence,
    counter_identity: BridgeIdentityEvidence,
    envelope_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEnvelopeIdentity {
    pub(super) fn query_observation_reporting(
        request_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
        evidence_binding_identity: BridgeIdentityEvidence,
        counter_identity: BridgeIdentityEvidence,
        envelope_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self {
            request_identity,
            causal_observation_anchor_identity,
            evidence_binding_identity,
            counter_identity,
            envelope_identity,
        }
    }

    pub(super) fn new(
        request_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
        evidence_binding_identity: BridgeIdentityEvidence,
        counter_identity: BridgeIdentityEvidence,
    ) -> Self {
        let envelope_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::Identity,
            &[
                evidence_part(&request_identity),
                evidence_part(&causal_observation_anchor_identity),
                evidence_part(&evidence_binding_identity),
                evidence_part(&counter_identity),
            ],
        );
        Self {
            request_identity,
            causal_observation_anchor_identity,
            evidence_binding_identity,
            counter_identity,
            envelope_identity,
        }
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn request_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.request_identity
    }

    pub fn causal_observation_anchor_for_reporting(&self) -> &str {
        self.causal_observation_anchor_identity.as_str()
    }

    pub fn causal_observation_anchor_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.causal_observation_anchor_identity
    }

    pub fn evidence_binding_digest_for_reporting(&self) -> &str {
        self.evidence_binding_identity.as_str()
    }

    pub fn evidence_binding_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.evidence_binding_identity
    }

    pub fn counter_for_reporting(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub fn counter_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.counter_identity
    }

    pub fn envelope_identity_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
    }

    pub fn envelope_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.envelope_identity
    }
}
