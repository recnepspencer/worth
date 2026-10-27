use super::*;

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_outcome_identity(
    kind: CausalInspectionAdmissionDecisionKind,
    subject_identity: &CausalInspectionAdmissionSubjectIdentity,
    decision_identity: &CausalInspectionAdmissionDecisionIdentity,
    trace_identity: &CausalInspectionDecisionTraceIdentity,
    receipt_identity: &CausalInspectionAdmissionReceiptIdentity,
) -> CausalInspectionOutcomeIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionOutcome)
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("subject"),
            subject_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("decision"),
            decision_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("trace"),
            trace_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("receipt"),
            receipt_identity.evidence_identity(),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_materialized_detail_identity(
    query_observation_identity: &WorthQueryEvidenceIdentity,
    advisory_reason: Option<&str>,
    readmission_proof: &CausalBridgeReadmissionProof,
    evidence_references: &[QueryCausalEvidenceReferenceArtifact],
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> CausalInspectionMaterializedDetailIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionMaterializedDetail)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_observation"),
            query_observation_identity,
        )
        .optional_shape(WorthQueryEvidenceTag::new("advisory"), advisory_reason)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("readmission"),
            readmission_proof.readmission_proof_identity(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("references"),
            evidence_references
                .iter()
                .map(QueryCausalEvidenceReferenceArtifact::reference_receipt_evidence_identity),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("redaction"),
            redaction_policy.as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("materialization"),
            materialization_policy.as_str(),
        )
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_denied_artifact_detail_identity(
    query_observation_identity: &WorthQueryEvidenceIdentity,
    result_shape_context_identity: &WorthQueryEvidenceIdentity,
    denial_reason: &str,
    bridge_denial_identity: Option<&WorthQueryEvidenceIdentity>,
    bridge_denial_kind: Option<BridgeCausalEnvelopeDenialKind>,
    bridge_denial_family: Option<BridgeCausalEvidenceFamily>,
) -> CausalInspectionDeniedArtifactDetailIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_observation"),
            query_observation_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("result_shape_context"),
            result_shape_context_identity,
        )
        .field_shape(WorthQueryEvidenceTag::new("reason"), denial_reason)
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_denial"),
            bridge_denial_identity,
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("bridge_denial_kind"),
            bridge_denial_kind
                .as_ref()
                .map(BridgeCausalEnvelopeDenialKind::as_str),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("bridge_denial_family"),
            bridge_denial_family
                .as_ref()
                .map(BridgeCausalEvidenceFamily::as_str),
        )
        .seal()
        .into()
}
