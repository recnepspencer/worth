use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod row_materialization;

use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

mod artifact;
mod hash;
mod identity;
mod inventory;
mod slots;

use super::super::inventory::CausalEvidenceFamily;
use super::super::materialization::QueryCausalInspectionArtifact;
use super::matrix_kind::CausalInspectionRepresentativeKind;
use artifact::{
    artifact_policy_digest, artifact_receipt_digest, evidence_reference_collection_digest,
    inspection_digest,
};
use hash::{row_digest, RowDigestParts};
use identity::RepresentativeCausalObservationAnchorDigest;
use slots::named_evidence_slots;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionRepresentativeRowDigestSet {
    kind: CausalInspectionRepresentativeKind,
    query_digest: String,
    query_observation_receipt_digest: String,
    causal_observation_anchor_digest: RepresentativeCausalObservationAnchorDigest,
    inspection_digest: Option<String>,
    artifact_digest: Option<String>,
    causal_envelope_digest: Option<String>,
    evidence_reference_collection_digest: Option<String>,
    relational_authority_digest: Option<String>,
    bridge_route_digest: Option<String>,
    bridge_evaluation_digest: Option<String>,
    bridge_source_materialization_digest: Option<String>,
    bridge_structural_digest: Option<String>,
    bridge_stream_digest: Option<String>,
    bridge_preview_digest: Option<String>,
    bridge_writeback_digest: Option<String>,
    bridge_replay_digest: Option<String>,
    signal_invalidation_digest: Option<String>,
    signal_evaluation_digest: Option<String>,
    signal_forensic_availability_digest: Option<String>,
    signal_replay_cursor_digest: Option<String>,
    signal_lineage_digest: Option<String>,
    signal_provenance_digest: Option<String>,
    replay_posture_digest: Option<String>,
    materialization_policy_digest: Option<String>,
    redaction_policy_digest: Option<String>,
    materialization_receipt_digest: Option<String>,
    counter_snapshot_digest: Option<String>,
    failure_digest: Option<String>,
    row_digest: String,
}

fn representative_query_digest(
    query_observation_receipt_digest: &str,
    causal_observation_anchor_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_representative_query_digest_v1",
    )
    .field_value(
        WorthQueryEvidenceTag::new("observation"),
        query_observation_receipt_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("anchor"),
        causal_observation_anchor_digest,
    )
    .seal()?
    .as_str()
    .to_string())
}
