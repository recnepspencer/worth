use super::scope::{certification_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_oracle_parity_lane_digest(
    declaration_digest: &str,
    contract_digest: &str,
    fact_set_digest: &str,
    receipt_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        certification_scope_encoder("projection_consumption_oracle_parity_lane_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("contract"), contract_digest)
            .field_shape(WorthQueryEvidenceTag::new("fact_set"), fact_set_digest)
            .field_shape(WorthQueryEvidenceTag::new("receipt"), receipt_digest),
    )
}

pub(crate) fn compose_oracle_comparison_row_digest(
    lane: &str,
    expected_digest: &str,
    actual_digest: &str,
    matched: bool,
) -> Result<String, IdentityDerivationDenial> {
    seal(
        certification_scope_encoder("projection_consumption_oracle_comparison_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("lane"), lane)
            .field_shape(WorthQueryEvidenceTag::new("expected"), expected_digest)
            .field_shape(WorthQueryEvidenceTag::new("actual"), actual_digest)
            .field_bool(WorthQueryEvidenceTag::new("match"), matched),
    )
}

pub(crate) fn compose_oracle_manifest_row_digest(
    lane: &str,
    lane_name: &str,
    owner: &str,
    sources: &[&str],
    forbidden_helpers: &[&str],
    fields: &[&str],
) -> Result<String, IdentityDerivationDenial> {
    seal(
        certification_scope_encoder("projection_consumption_oracle_manifest_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("lane"), lane)
            .field_shape(WorthQueryEvidenceTag::new("lane_name"), lane_name)
            .field_shape(WorthQueryEvidenceTag::new("owner"), owner)
            .field_value_sequence(
                WorthQueryEvidenceTag::new("source"),
                sources.iter().copied(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("forbidden_helper"),
                forbidden_helpers.iter().copied(),
            )
            .field_value_sequence(WorthQueryEvidenceTag::new("field"), fields.iter().copied()),
    )
}

pub(crate) fn compose_oracle_report_digest(
    comparison_row_digests: impl IntoIterator<Item = impl AsRef<str>>,
    manifest_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    let comparison_rows = comparison_row_digests
        .into_iter()
        .map(|digest| digest.as_ref().to_string())
        .collect::<Vec<_>>();
    seal(
        certification_scope_encoder("projection_consumption_oracle_report_v1")
            .field_value_sequence(
                WorthQueryEvidenceTag::new("comparison_row"),
                comparison_rows,
            )
            .field_shape(WorthQueryEvidenceTag::new("manifest"), manifest_digest),
    )
}
