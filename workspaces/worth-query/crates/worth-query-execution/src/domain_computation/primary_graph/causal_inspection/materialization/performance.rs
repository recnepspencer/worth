use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use worth_runtime_bridge::facade::{BridgeCausalEnvelopeDenial, BridgeCausalExplanationEnvelope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionPerformanceEnvelope {
    anchor_derivation_count: usize,
    evidence_reference_resolution_count: usize,
    admission_count: usize,
    bridge_envelope_assembly_count: usize,
    redaction_count: usize,
    materialization_count: usize,
    artifact_serialization_count: usize,
    bridge_binding_count: usize,
    bridge_lookup_count: usize,
    bridge_unindexed_scan_count: usize,
    materialized_detail_count: usize,
    performance_identity: WorthQueryEvidenceIdentity,
}

impl CausalInspectionPerformanceEnvelope {
    pub(super) fn for_bridge_envelope(
        envelope: &BridgeCausalExplanationEnvelope,
        redaction_count: usize,
    ) -> Self {
        let counters = envelope.counters();
        Self::new(
            1,
            1,
            1,
            1,
            redaction_count,
            1,
            1,
            counters.evidence_reference_count(),
            counters.bridge_retained_lookup_count(),
            counters.bridge_record_unindexed_scan_count(),
            counters.materialized_detail_count(),
        )
    }

    pub(super) fn for_denied_query() -> Self {
        Self::new(1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0)
    }

    pub(super) fn for_bridge_denial(denial: &BridgeCausalEnvelopeDenial) -> Self {
        let counters = denial.counters();
        Self::new(
            1,
            1,
            1,
            1,
            0,
            1,
            1,
            counters.evidence_reference_count(),
            counters.bridge_retained_lookup_count(),
            counters.bridge_record_unindexed_scan_count(),
            counters.materialized_detail_count(),
        )
    }

    fn new(
        anchor_derivation_count: usize,
        evidence_reference_resolution_count: usize,
        admission_count: usize,
        bridge_envelope_assembly_count: usize,
        redaction_count: usize,
        materialization_count: usize,
        artifact_serialization_count: usize,
        bridge_binding_count: usize,
        bridge_lookup_count: usize,
        bridge_unindexed_scan_count: usize,
        materialized_detail_count: usize,
    ) -> Self {
        let performance_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionPerformanceSnapshot,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("anchor"),
            anchor_derivation_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("references"),
            evidence_reference_resolution_count,
        )
        .field_usize(WorthQueryEvidenceTag::new("admission"), admission_count)
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_envelope"),
            bridge_envelope_assembly_count,
        )
        .field_usize(WorthQueryEvidenceTag::new("redaction"), redaction_count)
        .field_usize(
            WorthQueryEvidenceTag::new("materialization"),
            materialization_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("artifact_serialization"),
            artifact_serialization_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_bindings"),
            bridge_binding_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_lookups"),
            bridge_lookup_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_unindexed_scan"),
            bridge_unindexed_scan_count,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("materialized_detail"),
            materialized_detail_count,
        )
        .seal();
        Self {
            anchor_derivation_count,
            evidence_reference_resolution_count,
            admission_count,
            bridge_envelope_assembly_count,
            redaction_count,
            materialization_count,
            artifact_serialization_count,
            bridge_binding_count,
            bridge_lookup_count,
            bridge_unindexed_scan_count,
            materialized_detail_count,
            performance_identity,
        }
    }

    pub fn anchor_derivation_count(&self) -> usize {
        self.anchor_derivation_count
    }

    pub fn evidence_reference_resolution_count(&self) -> usize {
        self.evidence_reference_resolution_count
    }

    pub fn admission_count(&self) -> usize {
        self.admission_count
    }

    pub fn bridge_envelope_assembly_count(&self) -> usize {
        self.bridge_envelope_assembly_count
    }

    pub fn redaction_count(&self) -> usize {
        self.redaction_count
    }

    pub fn materialization_count(&self) -> usize {
        self.materialization_count
    }

    pub fn artifact_serialization_count(&self) -> usize {
        self.artifact_serialization_count
    }

    pub fn bridge_binding_count(&self) -> usize {
        self.bridge_binding_count
    }

    pub fn bridge_lookup_count(&self) -> usize {
        self.bridge_lookup_count
    }

    pub fn bridge_unindexed_scan_count(&self) -> usize {
        self.bridge_unindexed_scan_count
    }

    pub fn materialized_detail_count(&self) -> usize {
        self.materialized_detail_count
    }

    pub fn performance_for_reporting(&self) -> &str {
        self.performance_identity.as_str()
    }

    pub fn performance_digest(&self) -> &str {
        self.performance_for_reporting()
    }

    pub(super) fn performance_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.performance_identity
    }
}
