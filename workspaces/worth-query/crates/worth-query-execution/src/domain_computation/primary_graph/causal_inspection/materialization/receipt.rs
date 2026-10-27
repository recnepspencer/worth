use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::super::identity::CausalInspectionOutcomeIdentity;
use super::{
    CausalInspectionMaterializationPolicy, CausalInspectionPerformanceEnvelope,
    CausalInspectionRedactionPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalMaterializationReceipt {
    query_admission_identity: CausalInspectionOutcomeIdentity,
    bridge_envelope_identity: Option<WorthQueryEvidenceIdentity>,
    bridge_receipt_identity: Option<WorthQueryEvidenceIdentity>,
    policy_identity: WorthQueryEvidenceIdentity,
    performance_identity: WorthQueryEvidenceIdentity,
    materialization_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl CausalMaterializationReceipt {
    pub(super) fn new(
        query_admission_identity: &CausalInspectionOutcomeIdentity,
        bridge_envelope_identity: Option<&WorthQueryEvidenceIdentity>,
        bridge_receipt_identity: Option<&WorthQueryEvidenceIdentity>,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
        performance: &CausalInspectionPerformanceEnvelope,
        detail_identity: &WorthQueryEvidenceIdentity,
    ) -> Self {
        let policy_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalInspectionArtifact)
                .field_shape(WorthQueryEvidenceTag::new("role"), "materialization-policy")
                .field_shape(
                    WorthQueryEvidenceTag::new("redaction"),
                    redaction_policy.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("materialization"),
                    materialization_policy.as_str(),
                )
                .seal();
        let materialization_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalInspectionArtifact)
                .field_shape(WorthQueryEvidenceTag::new("role"), "materialization")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("query_admission"),
                    query_admission_identity.evidence_identity(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("bridge_envelope"),
                    bridge_envelope_identity,
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("bridge_receipt"),
                    bridge_receipt_identity,
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("policy"), &policy_identity)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("performance"),
                    performance.performance_identity(),
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("detail"), detail_identity)
                .seal();
        let receipt_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalInspectionArtifact)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "materialization-receipt",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("materialization"),
                    &materialization_identity,
                )
                .seal();
        Self {
            query_admission_identity: query_admission_identity.clone(),
            bridge_envelope_identity: bridge_envelope_identity.cloned(),
            bridge_receipt_identity: bridge_receipt_identity.cloned(),
            policy_identity,
            performance_identity: performance.performance_identity().clone(),
            materialization_identity,
            receipt_identity,
        }
    }

    pub fn query_admission_for_reporting(&self) -> &str {
        self.query_admission_identity.as_str()
    }

    pub fn query_admission_identity(&self) -> &CausalInspectionOutcomeIdentity {
        &self.query_admission_identity
    }

    pub fn bridge_envelope_for_reporting(&self) -> Option<&str> {
        self.bridge_envelope_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn bridge_envelope_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.bridge_envelope_identity.as_ref()
    }

    pub fn bridge_receipt_for_reporting(&self) -> Option<&str> {
        self.bridge_receipt_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn bridge_receipt_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.bridge_receipt_identity.as_ref()
    }

    pub fn policy_for_reporting(&self) -> &str {
        self.policy_identity.as_str()
    }

    pub fn policy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_identity
    }

    pub fn performance_for_reporting(&self) -> &str {
        self.performance_identity.as_str()
    }

    pub fn performance_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.performance_identity
    }

    pub fn materialization_for_reporting(&self) -> &str {
        self.materialization_identity.as_str()
    }

    pub fn materialization_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.materialization_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub(super) fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
