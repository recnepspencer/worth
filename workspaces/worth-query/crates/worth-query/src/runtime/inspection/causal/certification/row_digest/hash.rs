use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::matrix_kind::CausalInspectionRepresentativeKind;

pub(super) struct RowDigestParts<'a> {
    pub(super) kind: CausalInspectionRepresentativeKind,
    pub(super) query_digest: &'a str,
    pub(super) query_observation_receipt_digest: &'a str,
    pub(super) causal_observation_anchor_digest: &'a str,
    pub(super) inspection_digest: Option<&'a str>,
    pub(super) artifact_digest: Option<&'a str>,
    pub(super) causal_envelope_digest: Option<&'a str>,
    pub(super) evidence_reference_collection_digest: Option<&'a str>,
    pub(super) relational_authority_digest: Option<&'a str>,
    pub(super) bridge_route_digest: Option<&'a str>,
    pub(super) bridge_evaluation_digest: Option<&'a str>,
    pub(super) bridge_source_materialization_digest: Option<&'a str>,
    pub(super) bridge_structural_digest: Option<&'a str>,
    pub(super) bridge_stream_digest: Option<&'a str>,
    pub(super) bridge_preview_digest: Option<&'a str>,
    pub(super) bridge_writeback_digest: Option<&'a str>,
    pub(super) bridge_replay_digest: Option<&'a str>,
    pub(super) signal_invalidation_digest: Option<&'a str>,
    pub(super) signal_evaluation_digest: Option<&'a str>,
    pub(super) signal_forensic_availability_digest: Option<&'a str>,
    pub(super) signal_replay_cursor_digest: Option<&'a str>,
    pub(super) signal_lineage_digest: Option<&'a str>,
    pub(super) signal_provenance_digest: Option<&'a str>,
    pub(super) replay_posture_digest: Option<&'a str>,
    pub(super) materialization_policy_digest: Option<&'a str>,
    pub(super) redaction_policy_digest: Option<&'a str>,
    pub(super) materialization_receipt_digest: Option<&'a str>,
    pub(super) counter_snapshot_digest: Option<&'a str>,
    pub(super) failure_digest: Option<&'a str>,
}

pub(super) fn row_digest(parts: RowDigestParts<'_>) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_representative_row_digest_set_v1",
    )
    .field_shape(WorthQueryEvidenceTag::new("kind"), parts.kind.as_str())
    .field_value(WorthQueryEvidenceTag::new("query"), parts.query_digest)
    .field_value(
        WorthQueryEvidenceTag::new("observation"),
        parts.query_observation_receipt_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("anchor"),
        parts.causal_observation_anchor_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("inspection"),
        parts.inspection_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("artifact"),
        parts.artifact_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("envelope"),
        parts.causal_envelope_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("references"),
        parts.evidence_reference_collection_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("relational_authority"),
        parts.relational_authority_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_route"),
        parts.bridge_route_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_evaluation"),
        parts.bridge_evaluation_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_source"),
        parts.bridge_source_materialization_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_structural"),
        parts.bridge_structural_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_stream"),
        parts.bridge_stream_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_preview"),
        parts.bridge_preview_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_writeback"),
        parts.bridge_writeback_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("bridge_replay"),
        parts.bridge_replay_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_invalidation"),
        parts.signal_invalidation_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_evaluation"),
        parts.signal_evaluation_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_forensic_availability"),
        parts.signal_forensic_availability_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_replay_cursor"),
        parts.signal_replay_cursor_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_lineage"),
        parts.signal_lineage_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("signal_provenance"),
        parts.signal_provenance_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("replay_posture"),
        parts.replay_posture_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("materialization_policy"),
        parts.materialization_policy_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("redaction_policy"),
        parts.redaction_policy_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("materialization_receipt"),
        parts.materialization_receipt_digest,
    )
    .optional_identity(
        WorthQueryEvidenceTag::new("counter"),
        parts.counter_snapshot_digest,
    )
    .optional_identity(WorthQueryEvidenceTag::new("failure"), parts.failure_digest)
    .seal()?
    .as_str()
    .to_string())
}
