use super::*;

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_artifact_identity(
    kind: CausalInspectionArtifactKind,
    query_admission_identity: &CausalInspectionOutcomeIdentity,
    bridge_identity: Option<&WorthQueryEvidenceIdentity>,
    bridge_envelope: Option<&WorthQueryEvidenceIdentity>,
    receipt: &WorthQueryEvidenceIdentity,
    readmission_proof: Option<&WorthQueryEvidenceIdentity>,
    detail_identity: &WorthQueryEvidenceIdentity,
) -> CausalInspectionArtifactIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionArtifact)
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_admission"),
            query_admission_identity.evidence_identity(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_identity"),
            bridge_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_envelope"),
            bridge_envelope,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), receipt)
        .optional_evidence_identity(WorthQueryEvidenceTag::new("readmission"), readmission_proof)
        .field_evidence_identity(WorthQueryEvidenceTag::new("detail"), detail_identity)
        .seal()
        .into()
}

pub(in crate::domain_computation::primary_graph::causal_inspection) fn compose_causal_artifact_causal_identity(
    kind: CausalInspectionArtifactKind,
    query_admission_identity: &CausalInspectionOutcomeIdentity,
    query_observation_identity: &WorthQueryEvidenceIdentity,
    bridge_identity: Option<&WorthQueryEvidenceIdentity>,
    bridge_envelope: Option<&WorthQueryEvidenceIdentity>,
) -> CausalInspectionArtifactIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionArtifactIdentity)
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_admission"),
            query_admission_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_observation"),
            query_observation_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_identity"),
            bridge_identity,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_envelope"),
            bridge_envelope,
        )
        .seal()
        .into()
}
