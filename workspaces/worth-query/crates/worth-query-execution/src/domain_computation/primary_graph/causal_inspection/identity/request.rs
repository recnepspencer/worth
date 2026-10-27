use super::*;

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_inspection_target_identity(
    observation_target: &CausalObservationTargetHandle,
    result_shape_context: &CausalResultShapeContextHandle,
) -> CausalInspectionTargetIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionTarget)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("observation_target"),
            observation_target.identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("result_shape_context"),
            result_shape_context.identity().evidence_identity(),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_inspection_request_identity(
    anchor_digest: &CausalObservationAnchorDigest,
    reference_set_digest: &CausalEvidenceReferenceDigest,
    target_identity: &CausalInspectionTargetIdentity,
    explanation_family: CausalInspectionExplanationFamily,
    requested_richness: CausalInspectionRichness,
    requested_evidence_families: &[CausalEvidenceFamily],
    inspection_basis_digest: &str,
) -> CausalInspectionRequestIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionRequest)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            anchor_digest.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("reference_set"),
            reference_set_digest.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("target"),
            target_identity.evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("scoped_inspection_basis"),
            inspection_basis_digest,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            explanation_family.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("richness"),
            requested_richness.as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("evidence_families"),
            requested_evidence_families
                .iter()
                .map(CausalEvidenceFamily::as_str),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_inspection_request_failure_identity(
    kind: &str,
    message: &str,
    evidence: &[WorthQueryEvidenceIdentity],
) -> CausalInspectionRequestFailureIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionRequestFailure)
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind)
        .field_value(WorthQueryEvidenceTag::new("message"), message)
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("evidence"), evidence.iter())
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_admission_subject_identity(
    subject: &CausalInspectionAdmissionSubject,
) -> CausalInspectionAdmissionSubjectIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionAdmissionSubject)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("request"),
            subject.request_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            subject.anchor_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor_counters"),
            subject.anchor_counter_identity().evidence_identity(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("anchor_reference_families"),
            subject.anchor_reference_family_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("lower_runtime_families"),
            subject.lower_runtime_evidence_family_count(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query"),
            subject.query_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_observation"),
            subject.query_observation_identity().evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("inspection_reason"),
            subject.inspection_reason().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("observation_outcome"),
            subject.observation_outcome().as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("reference_set"),
            subject.reference_set_identity().evidence_identity(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("resolved_references"),
            subject.resolved_reference_count(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("missing_reference_families"),
            subject.missing_reference_family_count(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("resolved_evidence_families"),
            subject
                .resolved_evidence_families()
                .iter()
                .map(CausalEvidenceFamily::as_str),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("observation_target"),
            subject.observation_target_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("result_shape_context"),
            subject.result_shape_context_identity().evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("target"),
            subject.target_identity().evidence_identity(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            subject.explanation_family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("richness"),
            subject.requested_richness().as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("requested_evidence_families"),
            subject
                .requested_evidence_families()
                .iter()
                .map(CausalEvidenceFamily::as_str),
        )
        .seal()
        .into()
}
