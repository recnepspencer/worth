use super::super::identity::{
    compose_causal_admission_subject_identity, CausalInspectionAdmissionSubjectIdentity,
    CausalInspectionRequestIdentity, CausalInspectionTargetIdentity,
};
use super::super::inventory::{CausalEvidenceFamily, CausalEvidenceOwner};
use super::super::observation_identity::{
    CausalEvidenceReferenceDigest, CausalObservationAnchorCountersIdentity,
    CausalObservationAnchorDigest, CausalObservationQueryIdentity,
    CausalObservationReceiptIdentity, CausalObservationTargetHandle,
    CausalObservationTargetIdentity, CausalResultShapeContextHandle,
    CausalResultShapeContextIdentity,
};
use super::super::receipt_types::{CausalInspectionReason, CausalObservationOutcome};
use super::super::request::{
    CausalInspectionExplanationFamily, CausalInspectionRequest, CausalInspectionRichness,
};
use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceIdentityDerivationDenial, WorthQueryEvidenceScope,
};
use worth_runtime_bridge::facade::BridgeIdentityEvidence;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionAdmissionSubject {
    request_identity: CausalInspectionRequestIdentity,
    anchor_digest: CausalObservationAnchorDigest,
    anchor_counter_identity: CausalObservationAnchorCountersIdentity,
    anchor_reference_family_count: usize,
    lower_runtime_evidence_family_count: usize,
    query_identity: CausalObservationQueryIdentity,
    query_observation_identity: CausalObservationReceiptIdentity,
    inspection_reason: CausalInspectionReason,
    observation_outcome: CausalObservationOutcome,
    reference_set_digest: CausalEvidenceReferenceDigest,
    resolved_reference_count: usize,
    missing_reference_family_count: usize,
    resolved_evidence_families: Vec<CausalEvidenceFamily>,
    observation_target: CausalObservationTargetHandle,
    result_shape_context: CausalResultShapeContextHandle,
    target_identity: CausalInspectionTargetIdentity,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: Vec<CausalEvidenceFamily>,
    subject_identity: CausalInspectionAdmissionSubjectIdentity,
}

impl CausalInspectionAdmissionSubject {
    pub(in crate::domain_computation::primary_graph::causal_inspection) fn from_request(
        request: &CausalInspectionRequest,
    ) -> Result<Self, WorthQueryEvidenceIdentityDerivationDenial> {
        let requested_evidence_families = request.requested_evidence_families().to_vec();
        let anchor_digest = request.reference_set().anchor().anchor_digest().clone();
        let observation_receipt = request.reference_set().anchor().observation_receipt();
        let inspection_reason = request.reference_set().anchor().inspection_reason();
        let observation_outcome = observation_receipt.outcome();
        let anchor_counter_identity = request
            .reference_set()
            .anchor()
            .counters()
            .counter_identity()
            .clone();
        let anchor_reference_family_count = request
            .reference_set()
            .anchor()
            .counters()
            .reference_family_count();
        let lower_runtime_evidence_family_count = request
            .reference_set()
            .references()
            .iter()
            .filter(|reference| reference.owner() != CausalEvidenceOwner::Query)
            .map(|reference| reference.family())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let query_identity = observation_receipt.query_identity().clone();
        let query_observation_identity = observation_receipt.observation_receipt_identity().clone();
        let reference_set_digest = request.reference_set().reference_set_digest().clone();
        let resolved_reference_count = request.reference_set().receipt().resolved_reference_count();
        let missing_reference_family_count = request
            .reference_set()
            .receipt()
            .missing_reference_family_count();
        let resolved_evidence_families = request
            .reference_set()
            .references()
            .iter()
            .map(|reference| reference.family())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let observation_target = request.target().observation_target().clone();
        let result_shape_context = request.target().result_shape_context().clone();
        let target_identity = request.target().target_identity().clone();
        let mut subject = Self {
            request_identity: request.request_identity().clone(),
            anchor_digest,
            anchor_counter_identity,
            anchor_reference_family_count,
            lower_runtime_evidence_family_count,
            query_identity,
            query_observation_identity,
            inspection_reason,
            observation_outcome,
            reference_set_digest,
            resolved_reference_count,
            missing_reference_family_count,
            resolved_evidence_families,
            observation_target,
            result_shape_context,
            target_identity,
            explanation_family: request.explanation_family(),
            requested_richness: request.requested_richness(),
            requested_evidence_families,
            subject_identity: CausalInspectionAdmissionSubjectIdentity::from(
                WorthQueryEvidenceIdentity::compose(
                    WorthQueryEvidenceScope::CausalInspectionAdmissionSubject,
                )
                .seal()?,
            ),
        };
        subject.subject_identity = compose_causal_admission_subject_identity(&subject)?;
        Ok(subject)
    }

    pub fn request_digest(&self) -> &str {
        self.request_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn request_identity(
        &self,
    ) -> &CausalInspectionRequestIdentity {
        &self.request_identity
    }

    pub fn anchor_for_reporting(&self) -> &str {
        self.anchor_digest.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn anchor_identity(
        &self,
    ) -> &CausalObservationAnchorDigest {
        &self.anchor_digest
    }

    pub fn anchor_counter_snapshot(&self) -> &str {
        self.anchor_counter_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn anchor_counter_identity(
        &self,
    ) -> &CausalObservationAnchorCountersIdentity {
        &self.anchor_counter_identity
    }

    pub fn anchor_reference_family_count(&self) -> usize {
        self.anchor_reference_family_count
    }

    pub fn lower_runtime_evidence_family_count(&self) -> usize {
        self.lower_runtime_evidence_family_count
    }

    pub fn query_for_reporting(&self) -> &str {
        self.query_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn query_identity(
        &self,
    ) -> &CausalObservationQueryIdentity {
        &self.query_identity
    }

    pub fn query_observation_digest(&self) -> &str {
        self.query_observation_identity.as_str()
    }

    pub fn inspection_reason(&self) -> CausalInspectionReason {
        self.inspection_reason
    }

    pub fn observation_outcome(&self) -> CausalObservationOutcome {
        self.observation_outcome
    }

    pub fn reference_set_digest(&self) -> &str {
        self.reference_set_digest.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn reference_set_identity(
        &self,
    ) -> &CausalEvidenceReferenceDigest {
        &self.reference_set_digest
    }

    pub fn resolved_reference_count(&self) -> usize {
        self.resolved_reference_count
    }

    pub fn missing_reference_family_count(&self) -> usize {
        self.missing_reference_family_count
    }

    pub fn resolved_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.resolved_evidence_families
    }

    pub fn observation_target_for_reporting(&self) -> &str {
        self.observation_target.identity().as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn observation_target_identity(
        &self,
    ) -> &CausalObservationTargetIdentity {
        self.observation_target.identity()
    }

    pub fn result_shape_context_for_reporting(&self) -> &str {
        self.result_shape_context.identity().as_str()
    }

    pub fn target_for_reporting(&self) -> &str {
        self.target_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn target_identity(
        &self,
    ) -> &CausalInspectionTargetIdentity {
        &self.target_identity
    }

    pub fn explanation_family(&self) -> CausalInspectionExplanationFamily {
        self.explanation_family
    }

    pub fn requested_richness(&self) -> CausalInspectionRichness {
        self.requested_richness
    }

    pub fn requested_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.requested_evidence_families
    }

    pub fn subject_for_reporting(&self) -> &str {
        self.subject_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn query_observation_identity(
        &self,
    ) -> &CausalObservationReceiptIdentity {
        &self.query_observation_identity
    }

    pub fn query_observation_for_reporting(&self) -> &str {
        self.query_observation_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn query_observation_evidence_identity(
        &self,
    ) -> &crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity {
        self.query_observation_identity.evidence_identity()
    }

    pub fn query_observation_bridge_evidence_identity(&self) -> BridgeIdentityEvidence {
        self.query_observation_evidence_identity()
            .bridge_evidence_identity()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn result_shape_context_identity(
        &self,
    ) -> &CausalResultShapeContextIdentity {
        self.result_shape_context.identity()
    }

    pub(in crate::domain_computation::primary_graph::causal_inspection) fn subject_identity(
        &self,
    ) -> &CausalInspectionAdmissionSubjectIdentity {
        &self.subject_identity
    }
}
