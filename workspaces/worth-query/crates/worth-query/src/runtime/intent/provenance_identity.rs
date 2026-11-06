use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

pub(super) fn intent_execution_provenance_chain_identity(
    provenance: IntentExecutionProvenanceIdentityParts<'_>,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::IntentExecutionProvenanceChain)
        .field_shape(WorthQueryEvidenceTag::new("family"), provenance.family)
        .field_shape(
            WorthQueryEvidenceTag::new("entrypoint"),
            provenance.entrypoint,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("seam"),
            provenance.execution_seam,
        )
        .field_value(
            WorthQueryEvidenceTag::new("admission_decision_digest"),
            provenance.admission_decision_digest,
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_handoff_digest"),
            provenance.execution_handoff_digest,
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_binding_digest"),
            provenance.execution_binding_digest,
        )
        .field_value(
            WorthQueryEvidenceTag::new("execution_outcome_digest"),
            provenance.execution_outcome_digest,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot_token"),
            provenance.snapshot_evidence_identity,
        )
        .seal()?)
}

pub(super) struct IntentExecutionProvenanceIdentityParts<'a> {
    pub(super) family: &'a str,
    pub(super) entrypoint: &'a str,
    pub(super) execution_seam: &'a str,
    pub(super) admission_decision_digest: &'a str,
    pub(super) execution_handoff_digest: &'a str,
    pub(super) execution_binding_digest: &'a str,
    pub(super) execution_outcome_digest: &'a str,
    pub(super) snapshot_evidence_identity: &'a WorthQueryEvidenceIdentity,
}
