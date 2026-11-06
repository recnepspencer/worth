use super::*;
use crate::runtime::WorthQueryRuntimeAsyncResultStateKind;

pub(in crate::runtime) fn shared_read_unpublished_causality_identity(
    view_name: &str,
    snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "shared_read_unpublished_causality_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("view"), view_name)
        .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
        .seal()?)
}

pub(in crate::runtime) fn shared_read_republishing_causality_identity(
    view_name: &str,
    kind: WorthQueryRuntimeAsyncResultStateKind,
    snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "shared_read_republishing_causality_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("view"), view_name)
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
        .seal()?)
}

pub(in crate::runtime) fn shared_read_bind_retained_artifact_label_identity(
    view_name: &str,
    snapshot_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
    Ok(worth_query_evidence_identity(WorthQueryEvidenceScope::SharedReadGeneration)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "shared_read_bind_retained_artifact_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("view"), view_name)
        .field_evidence_identity(WorthQueryEvidenceTag::new("snapshot"), snapshot_identity)
        .seal()?)
}
