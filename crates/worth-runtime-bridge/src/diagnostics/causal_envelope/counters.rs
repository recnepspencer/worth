use super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, value_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEnvelopeCounters {
    evidence_reference_count: usize,
    lower_runtime_family_count: usize,
    bridge_retained_lookup_count: usize,
    retained_bridge_binding_count: usize,
    external_authority_reference_count: usize,
    materialized_detail_count: usize,
    missing_bridge_record_count: usize,
    bridge_record_unindexed_scan_count: usize,
    retained_native_observation_binding_count: usize,
    counter_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEnvelopeCounters {
    pub(super) fn query_observation_reporting(
        counts: [usize; 8],
        counter_identity: BridgeIdentityEvidence,
    ) -> Self {
        Self {
            evidence_reference_count: counts[0],
            lower_runtime_family_count: counts[1],
            bridge_retained_lookup_count: counts[2],
            retained_bridge_binding_count: counts[3],
            external_authority_reference_count: counts[4],
            materialized_detail_count: counts[5],
            missing_bridge_record_count: counts[6],
            bridge_record_unindexed_scan_count: counts[7],
            retained_native_observation_binding_count: 0,
            counter_identity,
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0)
    }

    pub(crate) fn new(
        evidence_reference_count: usize,
        lower_runtime_family_count: usize,
        bridge_retained_lookup_count: usize,
        retained_bridge_binding_count: usize,
        external_authority_reference_count: usize,
        materialized_detail_count: usize,
        missing_bridge_record_count: usize,
    ) -> Self {
        let evidence_reference_count_text = evidence_reference_count.to_string();
        let lower_runtime_family_count_text = lower_runtime_family_count.to_string();
        let bridge_retained_lookup_count_text = bridge_retained_lookup_count.to_string();
        let retained_bridge_binding_count_text = retained_bridge_binding_count.to_string();
        let external_authority_reference_count_text =
            external_authority_reference_count.to_string();
        let materialized_detail_count_text = materialized_detail_count.to_string();
        let missing_bridge_record_count_text = missing_bridge_record_count.to_string();
        let counter_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::Counters,
            &[
                value_part(evidence_reference_count_text.as_str()),
                value_part(lower_runtime_family_count_text.as_str()),
                value_part(bridge_retained_lookup_count_text.as_str()),
                value_part(retained_bridge_binding_count_text.as_str()),
                value_part(external_authority_reference_count_text.as_str()),
                value_part(materialized_detail_count_text.as_str()),
                value_part(missing_bridge_record_count_text.as_str()),
                value_part("0"),
            ],
        );
        Self {
            evidence_reference_count,
            lower_runtime_family_count,
            bridge_retained_lookup_count,
            retained_bridge_binding_count,
            external_authority_reference_count,
            materialized_detail_count,
            missing_bridge_record_count,
            bridge_record_unindexed_scan_count: 0,
            retained_native_observation_binding_count: 0,
            counter_identity,
        }
    }

    pub fn evidence_reference_count(&self) -> usize {
        self.evidence_reference_count
    }

    pub(super) fn with_native_bindings(mut self, count: usize) -> Self {
        self.counter_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::NativeCounters,
            &[
                super::evidence_part(&self.counter_identity),
                value_part(&count.to_string()),
            ],
        );
        self.retained_native_observation_binding_count = count;
        self
    }

    pub fn retained_native_observation_binding_count(&self) -> usize {
        self.retained_native_observation_binding_count
    }

    pub fn lower_runtime_family_count(&self) -> usize {
        self.lower_runtime_family_count
    }

    pub fn bridge_retained_lookup_count(&self) -> usize {
        self.bridge_retained_lookup_count
    }

    pub fn retained_bridge_binding_count(&self) -> usize {
        self.retained_bridge_binding_count
    }

    pub fn external_authority_reference_count(&self) -> usize {
        self.external_authority_reference_count
    }

    pub fn materialized_detail_count(&self) -> usize {
        self.materialized_detail_count
    }

    pub fn missing_bridge_record_count(&self) -> usize {
        self.missing_bridge_record_count
    }

    pub fn bridge_record_unindexed_scan_count(&self) -> usize {
        self.bridge_record_unindexed_scan_count
    }

    pub fn counter_for_reporting(&self) -> &str {
        self.counter_identity.as_str()
    }

    pub fn counter_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.counter_identity
    }
}
