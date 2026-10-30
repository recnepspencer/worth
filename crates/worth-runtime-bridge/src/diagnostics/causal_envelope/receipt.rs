use super::counters::BridgeCausalEnvelopeCounters;
use super::identity::BridgeCausalEnvelopeIdentity;
use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEnvelopeReceipt {
    envelope_identity: BridgeIdentityEvidence,
    envelope_digest_identity: BridgeIdentityEvidence,
    counter_identity: BridgeIdentityEvidence,
    receipt_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEnvelopeReceipt {
    pub(super) fn query_observation_reporting(
        identity: &BridgeCausalEnvelopeIdentity,
        envelope_identity: &BridgeIdentityEvidence,
        counters: &BridgeCausalEnvelopeCounters,
        receipt_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self {
            envelope_identity: identity.envelope_evidence_identity().clone(),
            envelope_digest_identity: envelope_identity.clone(),
            counter_identity: counters.counter_evidence_identity().clone(),
            receipt_identity,
        }
    }

    pub(super) fn new(
        identity: &BridgeCausalEnvelopeIdentity,
        envelope_identity: &BridgeIdentityEvidence,
        counters: &BridgeCausalEnvelopeCounters,
    ) -> Self {
        let receipt_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::Receipt,
            &[
                evidence_part(identity.envelope_evidence_identity()),
                evidence_part(envelope_identity),
                evidence_part(counters.counter_evidence_identity()),
            ],
        );
        Self {
            envelope_identity: identity.envelope_evidence_identity().clone(),
            envelope_digest_identity: envelope_identity.clone(),
            counter_identity: counters.counter_evidence_identity().clone(),
            receipt_identity,
        }
    }

    pub fn envelope_identity_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
    }

    pub fn envelope_identity_evidence(&self) -> &BridgeIdentityEvidence {
        &self.envelope_identity
    }

    pub fn envelope_for_reporting(&self) -> &str {
        self.envelope_digest_identity.as_str()
    }

    pub fn envelope_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.envelope_digest_identity
    }

    pub fn counter_for_reporting(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub fn counter_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.counter_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.receipt_identity
    }
}
