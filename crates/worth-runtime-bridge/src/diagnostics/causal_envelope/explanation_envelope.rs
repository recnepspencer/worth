use super::assembly::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalInspectionAdmissionSummaryKind,
};
use super::binding::BridgeCausalEvidenceBinding;
use super::counters::BridgeCausalEnvelopeCounters;
use super::identity::BridgeCausalEnvelopeIdentity;
use super::receipt::BridgeCausalEnvelopeReceipt;
use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalExplanationEnvelope {
    identity: BridgeCausalEnvelopeIdentity,
    admission_summary_kind: BridgeCausalInspectionAdmissionSummaryKind,
    admission_summary_identity: BridgeIdentityEvidence,
    request_identity: BridgeIdentityEvidence,
    causal_observation_anchor_identity: BridgeIdentityEvidence,
    bindings: Box<[BridgeCausalEvidenceBinding]>,
    counters: BridgeCausalEnvelopeCounters,
    receipt: BridgeCausalEnvelopeReceipt,
    envelope_identity: BridgeIdentityEvidence,
    retained_native: super::retained_source::RetainedNativeObservation,
}

impl BridgeCausalExplanationEnvelope {
    pub(super) fn new(
        request: BridgeCausalEnvelopeAssemblyRequest,
        bindings: Vec<BridgeCausalEvidenceBinding>,
        counters: BridgeCausalEnvelopeCounters,
    ) -> Self {
        let binding_parts = bindings
            .iter()
            .map(BridgeCausalEvidenceBinding::binding_evidence_identity)
            .collect::<Vec<_>>();
        let evidence_binding_parts = binding_parts.iter().map(evidence_part).collect::<Vec<_>>();
        let evidence_binding_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::BindingSet,
            &evidence_binding_parts,
        );
        let request_identity = request.request_evidence_identity().clone();
        let causal_observation_anchor_identity = request
            .admission_summary()
            .causal_observation_anchor_identity()
            .clone();
        let identity = BridgeCausalEnvelopeIdentity::new(
            request_identity.clone(),
            causal_observation_anchor_identity.clone(),
            evidence_binding_identity,
            counters.counter_evidence_identity().clone(),
        );
        let mut envelope_parts = Vec::with_capacity(bindings.len() + 4);
        envelope_parts.push(evidence_part(identity.envelope_evidence_identity()));
        envelope_parts.push(evidence_part(&request_identity));
        envelope_parts.push(evidence_part(&causal_observation_anchor_identity));
        envelope_parts.extend(binding_parts.iter().map(evidence_part));
        envelope_parts.push(evidence_part(counters.counter_evidence_identity()));
        let envelope_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::ExplanationEnvelope,
            &envelope_parts,
        );
        let receipt = BridgeCausalEnvelopeReceipt::new(&identity, &envelope_identity, &counters);
        Self {
            identity,
            admission_summary_kind: request.admission_summary().kind(),
            admission_summary_identity: request
                .admission_summary()
                .summary_evidence_identity()
                .clone(),
            request_identity,
            causal_observation_anchor_identity,
            bindings: bindings.into_boxed_slice(),
            counters,
            receipt,
            envelope_identity,
            retained_native: Default::default(),
        }
    }

    pub fn identity(&self) -> &BridgeCausalEnvelopeIdentity {
        &self.identity
    }

    pub(super) fn retain_native(
        mut self,
        observation: crate::conditional_execution::BridgeConditionalCausalObservation,
    ) -> Self {
        self.retained_native.0 = Some(observation);
        self
    }

    /// Confirms exact retained custody; descriptive digest equality cannot do so.
    pub fn retains_native_observation(
        &self,
        observation: &crate::conditional_execution::BridgeConditionalCausalObservation,
    ) -> bool {
        self.retained_native
            .0
            .as_ref()
            .is_some_and(|retained| retained.same_observation(observation))
    }

    pub fn admission_summary_kind(&self) -> BridgeCausalInspectionAdmissionSummaryKind {
        self.admission_summary_kind
    }

    pub fn admission_summary_for_reporting(&self) -> &str {
        self.admission_summary_identity.as_str()
    }

    pub fn admission_summary_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.admission_summary_identity
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

    pub fn bindings(&self) -> &[BridgeCausalEvidenceBinding] {
        &self.bindings
    }

    pub fn counters(&self) -> &BridgeCausalEnvelopeCounters {
        &self.counters
    }

    pub fn receipt(&self) -> &BridgeCausalEnvelopeReceipt {
        &self.receipt
    }

    pub fn envelope_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
    }

    pub fn envelope_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.envelope_identity
    }
}
