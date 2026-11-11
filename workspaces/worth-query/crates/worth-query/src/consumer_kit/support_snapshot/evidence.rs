use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::row::WorthQuerySupportSnapshotRow;
use super::schema::WorthQuerySupportSnapshotSchemaVersion;

pub(crate) fn support_snapshot_row_identity(
    schema_version: WorthQuerySupportSnapshotSchemaVersion,
    row: &WorthQuerySupportSnapshotRow,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportSnapshotRow)
            .field_shape(
                WorthQueryEvidenceTag::new("schema_version"),
                schema_version.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("surface"), row.surface())
            .optional_shape(
                WorthQueryEvidenceTag::new("facade_family"),
                row.facade_family(),
            )
            .field_shape(WorthQueryEvidenceTag::new("status"), row.status())
            .field_shape(
                WorthQueryEvidenceTag::new("teaching_posture"),
                row.teaching_posture(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("owner_milestone"),
                row.owner_milestone(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("extension_rule"),
                row.extension_rule(),
            )
            .field_bool(
                WorthQueryEvidenceTag::new("parallel_api_forbidden"),
                row.parallel_api_forbidden(),
            )
            .field_bool(
                WorthQueryEvidenceTag::new("admission_fail_closed"),
                row.admission_fail_closed(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("support_contract_digest"),
                row.support_contract_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("live_row_digest"),
                row.live_row_digest(),
            )
            .seal()?,
    )
}

pub(crate) fn support_snapshot_document_identity<'a>(
    schema_version: WorthQuerySupportSnapshotSchemaVersion,
    schema_identity: &WorthQueryEvidenceIdentity,
    backend_posture: &str,
    source_matrix_digest: &str,
    rows: impl IntoIterator<Item = &'a WorthQuerySupportSnapshotRow>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let rows = rows.into_iter().collect::<Vec<_>>();
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportSnapshotDocument)
            .field_shape(
                WorthQueryEvidenceTag::new("schema_version"),
                schema_version.as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("schema_identity"),
                schema_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("backend_posture"),
                backend_posture,
            )
            .field_value(
                WorthQueryEvidenceTag::new("source_matrix_digest"),
                source_matrix_digest,
            )
            .field_usize(WorthQueryEvidenceTag::new("row_count"), rows.len())
            .field_value_sequence(
                WorthQueryEvidenceTag::new("snapshot_row_digest"),
                rows.iter().map(|row| row.snapshot_row_digest()),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("live_row_digest"),
                rows.iter().map(|row| row.live_row_digest()),
            )
            .seal()?,
    )
}
