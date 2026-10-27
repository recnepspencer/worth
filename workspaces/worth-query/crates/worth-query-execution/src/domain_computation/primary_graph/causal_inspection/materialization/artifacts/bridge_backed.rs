mod artifact_union;

pub use artifact_union::QueryCausalInspectionArtifact;

use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use worth_runtime_bridge::facade::{
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalExplanationEnvelope,
};

use super::super::super::identity::{
    compose_bridge_causal_envelope_identity, compose_bridge_causal_envelope_receipt_identity,
    compose_bridge_causal_explanation_envelope_identity, CausalInspectionArtifactIdentity,
    CausalInspectionOutcomeIdentity,
};
use super::super::super::observation_identity::{
    CausalObservationReceiptIdentity, CausalResultShapeContextIdentity,
};
use super::super::{
    CausalBridgeReadmissionProof, CausalInspectionArtifactKind,
    CausalInspectionBoundaryEnvelopeCategory, CausalInspectionPerformanceEnvelope,
    CausalInspectionRedactionPolicy, CausalMaterializationReceipt,
    QueryCausalTemporalAsyncExplanation,
};
use super::built::BuiltBridgeBackedArtifact;
use super::denied::DeniedQueryCausalInspectionArtifact;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryCausalEvidenceReferenceArtifact {
    owner: BridgeCausalEvidenceOwner,
    family: BridgeCausalEvidenceFamily,
    reference_identity: WorthQueryEvidenceIdentity,
    binding_identity: WorthQueryEvidenceIdentity,
    retained_record_identity: Option<WorthQueryEvidenceIdentity>,
    detail_redacted: bool,
    reference_receipt_identity: WorthQueryEvidenceIdentity,
}

impl QueryCausalEvidenceReferenceArtifact {
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) fn from_bridge_binding(
        binding: &BridgeCausalEvidenceBinding,
        redaction_policy: CausalInspectionRedactionPolicy,
    ) -> Self {
        let detail_redacted = redaction_policy == CausalInspectionRedactionPolicy::DigestOnly;
        let retained_record_identity = if detail_redacted {
            None
        } else {
            binding.retained_record_evidence_identity()
        };
        let bridge_reference_identity = binding.reference_evidence_identity();
        let bridge_binding_identity = binding.binding_evidence_identity();
        let reference_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "bridge-causal-evidence-reference",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("owner"),
            binding.owner().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            binding.family().as_str(),
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("reference"),
            &bridge_reference_identity,
        )
        .seal();
        let binding_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "bridge-causal-evidence-binding",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("binding_class"),
            bridge_causal_evidence_binding_class_label(binding.binding_class()),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("reference"), &reference_identity)
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("binding"),
            &bridge_binding_identity,
        )
        .seal();
        let retained_record_identity = retained_record_identity.as_ref().map(|retained_record| {
            WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), "retained-bridge-record")
            .field_shape(
                WorthQueryEvidenceTag::new("owner"),
                binding.owner().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                binding.family().as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("reference"), &reference_identity)
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("retained"),
                retained_record,
            )
            .seal()
        });
        let reference_receipt_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("owner"),
            binding.owner().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            binding.family().as_str(),
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("reference"), &reference_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("retained"),
            retained_record_identity.as_ref(),
        )
        .field_bool(WorthQueryEvidenceTag::new("redacted"), detail_redacted)
        .seal();
        Self {
            owner: binding.owner(),
            family: binding.family(),
            reference_identity,
            binding_identity,
            retained_record_identity,
            detail_redacted,
            reference_receipt_identity,
        }
    }

    pub fn owner(&self) -> &str {
        self.owner.as_str()
    }

    pub fn owner_kind(&self) -> BridgeCausalEvidenceOwner {
        self.owner
    }

    pub fn family(&self) -> &str {
        self.family.as_str()
    }

    pub fn family_kind(&self) -> BridgeCausalEvidenceFamily {
        self.family
    }

    pub fn reference_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.reference_identity
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn retained_record_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.retained_record_identity.as_ref()
    }

    pub fn reference_evidence_for_reporting(&self) -> &str {
        self.reference_identity.as_str()
    }

    pub fn binding_for_reporting(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn retained_record_for_reporting(&self) -> Option<&str> {
        self.retained_record_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn detail_redacted(&self) -> bool {
        self.detail_redacted
    }

    pub fn reference_for_reporting(&self) -> &str {
        self.reference_receipt_identity.as_str()
    }

    pub fn reference_receipt_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.reference_receipt_identity
    }
}

fn bridge_causal_evidence_binding_class_label(
    binding_class: BridgeCausalEvidenceBindingClass,
) -> &'static str {
    match binding_class {
        BridgeCausalEvidenceBindingClass::RetainedBridgeRecord => "retained_bridge_record",
        BridgeCausalEvidenceBindingClass::ExternalAuthorityReference => {
            "external_authority_reference"
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedQueryCausalInspectionArtifact {
    query_admission_identity: CausalInspectionOutcomeIdentity,
    query_observation_identity: CausalObservationReceiptIdentity,
    result_shape_context_identity: CausalResultShapeContextIdentity,
    bridge_identity: WorthQueryEvidenceIdentity,
    bridge_envelope_identity: WorthQueryEvidenceIdentity,
    bridge_receipt_identity: WorthQueryEvidenceIdentity,
    boundary_categories: Vec<CausalInspectionBoundaryEnvelopeCategory>,
    evidence_references: Vec<QueryCausalEvidenceReferenceArtifact>,
    temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
    performance: CausalInspectionPerformanceEnvelope,
    receipt: CausalMaterializationReceipt,
    readmission_proof: CausalBridgeReadmissionProof,
    causal_identity: CausalInspectionArtifactIdentity,
    artifact_identity: CausalInspectionArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryQueryCausalInspectionArtifact {
    query_advisory_identity: CausalInspectionOutcomeIdentity,
    query_observation_identity: CausalObservationReceiptIdentity,
    result_shape_context_identity: CausalResultShapeContextIdentity,
    advisory_reason: String,
    bridge_identity: WorthQueryEvidenceIdentity,
    bridge_envelope_identity: WorthQueryEvidenceIdentity,
    bridge_receipt_identity: WorthQueryEvidenceIdentity,
    boundary_categories: Vec<CausalInspectionBoundaryEnvelopeCategory>,
    evidence_references: Vec<QueryCausalEvidenceReferenceArtifact>,
    temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
    performance: CausalInspectionPerformanceEnvelope,
    receipt: CausalMaterializationReceipt,
    readmission_proof: CausalBridgeReadmissionProof,
    causal_identity: CausalInspectionArtifactIdentity,
    artifact_identity: CausalInspectionArtifactIdentity,
}

impl AdmittedQueryCausalInspectionArtifact {
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) fn from_parts(
        query_admission_identity: &CausalInspectionOutcomeIdentity,
        query_observation_identity: &CausalObservationReceiptIdentity,
        result_shape_context_identity: &CausalResultShapeContextIdentity,
        envelope: &BridgeCausalExplanationEnvelope,
        temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
        built: BuiltBridgeBackedArtifact,
    ) -> Self {
        let bridge_identity = compose_bridge_causal_envelope_identity(envelope.identity());
        let bridge_envelope_identity =
            compose_bridge_causal_explanation_envelope_identity(envelope);
        let bridge_receipt_identity =
            compose_bridge_causal_envelope_receipt_identity(envelope.receipt());
        Self {
            query_admission_identity: query_admission_identity.clone(),
            query_observation_identity: query_observation_identity.clone(),
            result_shape_context_identity: result_shape_context_identity.clone(),
            bridge_identity,
            bridge_envelope_identity,
            bridge_receipt_identity,
            boundary_categories: built.boundary_categories,
            evidence_references: built.evidence_references,
            temporal_async_explanation,
            performance: built.performance,
            receipt: built.receipt,
            readmission_proof: built.readmission_proof,
            causal_identity: built.causal_identity,
            artifact_identity: built.artifact_identity,
        }
    }

    pub fn query_admission_for_reporting(&self) -> &str {
        self.query_admission_identity.as_str()
    }

    pub fn query_admission_identity(&self) -> &CausalInspectionOutcomeIdentity {
        &self.query_admission_identity
    }

    pub fn query_observation_for_reporting(&self) -> &str {
        self.query_observation_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn query_observation_identity(
        &self,
    ) -> &CausalObservationReceiptIdentity {
        &self.query_observation_identity
    }

    pub fn result_shape_context_for_reporting(&self) -> &str {
        self.result_shape_context_identity.as_str()
    }
}

impl AdvisoryQueryCausalInspectionArtifact {
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) fn from_parts(
        query_advisory_identity: &CausalInspectionOutcomeIdentity,
        query_observation_identity: &CausalObservationReceiptIdentity,
        result_shape_context_identity: &CausalResultShapeContextIdentity,
        advisory_reason: String,
        envelope: &BridgeCausalExplanationEnvelope,
        temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
        built: BuiltBridgeBackedArtifact,
    ) -> Self {
        let bridge_identity = compose_bridge_causal_envelope_identity(envelope.identity());
        let bridge_envelope_identity =
            compose_bridge_causal_explanation_envelope_identity(envelope);
        let bridge_receipt_identity =
            compose_bridge_causal_envelope_receipt_identity(envelope.receipt());
        Self {
            query_advisory_identity: query_advisory_identity.clone(),
            query_observation_identity: query_observation_identity.clone(),
            result_shape_context_identity: result_shape_context_identity.clone(),
            advisory_reason,
            bridge_identity,
            bridge_envelope_identity,
            bridge_receipt_identity,
            boundary_categories: built.boundary_categories,
            evidence_references: built.evidence_references,
            temporal_async_explanation,
            performance: built.performance,
            receipt: built.receipt,
            readmission_proof: built.readmission_proof,
            causal_identity: built.causal_identity,
            artifact_identity: built.artifact_identity,
        }
    }

    pub fn query_advisory_for_reporting(&self) -> &str {
        self.query_advisory_identity.as_str()
    }

    pub fn query_observation_for_reporting(&self) -> &str {
        self.query_observation_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn query_observation_identity(
        &self,
    ) -> &CausalObservationReceiptIdentity {
        &self.query_observation_identity
    }

    pub fn result_shape_context_for_reporting(&self) -> &str {
        self.result_shape_context_identity.as_str()
    }

    pub fn advisory_reason(&self) -> &str {
        &self.advisory_reason
    }
}
