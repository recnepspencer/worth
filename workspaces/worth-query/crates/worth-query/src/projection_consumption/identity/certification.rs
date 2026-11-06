use super::scope::{
    certification_scope_encoder, compose_certification_sequence_digest,
    compose_labeled_entry_digest, seal,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::contracts::MaterializedProjectionContract;
use crate::projection_consumption::source::ProjectionSourceFamily;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_certification_row_digest(
    identity_family: &'static str,
    entries: &[(&'static str, &str)],
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_labeled_entry_digest(identity_family, entries)?)
}

pub(crate) fn compose_certification_counter_snapshot_digest(
    declared: usize,
    admitted: usize,
    extracted: usize,
    row_width: usize,
    evidence_width: usize,
    authority_reopen: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_counter_snapshot_v1")
            .field_usize(WorthQueryEvidenceTag::new("declared"), declared)
            .field_usize(WorthQueryEvidenceTag::new("admitted"), admitted)
            .field_usize(WorthQueryEvidenceTag::new("extracted"), extracted)
            .field_usize(WorthQueryEvidenceTag::new("row_width"), row_width)
            .field_usize(WorthQueryEvidenceTag::new("evidence_width"), evidence_width)
            .field_usize(
                WorthQueryEvidenceTag::new("authority_reopen"),
                authority_reopen,
            ),
    )?)
}

pub(crate) fn compose_slope_digest(
    label: &str,
    scale_parts: impl IntoIterator<Item = (usize, Vec<(&'static str, String)>)>,
) -> String {
    let entries = scale_parts
        .into_iter()
        .map(|(scale, parts)| {
            let mut encoder = certification_scope_encoder("projection_consumption_slope_scale_v1")
                .field_shape(WorthQueryEvidenceTag::new("label"), label)
                .field_usize(WorthQueryEvidenceTag::new("rows"), scale);
            for (tag, value) in parts {
                encoder = encoder.field_shape(WorthQueryEvidenceTag::new(tag), value.as_str());
            }
            seal(encoder)
        })
        .collect::<Vec<_>>();
    compose_certification_sequence_digest(
        "projection_consumption_slope_v1",
        "scale",
        entries.iter().map(String::as_str),
    )
}

fn compose_certified_source_reference_entry_digest(
    label: &str,
    identity: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_certified_source_reference_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("label"), label)
            .field_shape(WorthQueryEvidenceTag::new("identity"), identity),
    )?)
}

pub(crate) fn compose_certified_source_digest(
    contract: &MaterializedProjectionContract,
) -> Result<String, IdentityDerivationDenial> {
    let source_references = contract
        .source_reference_identities()
        .iter()
        .map(|identity| {
            compose_certified_source_reference_entry_digest(identity.label(), identity.identity())
        })
        .collect::<Vec<_>>();
    Ok(seal(
        certification_scope_encoder("projection_consumption_certified_source_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                contract.source_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("identity"),
                contract.source_identity(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("source_reference"),
                source_references,
            ),
    )?)
}

pub(crate) fn compose_certified_source_receipt_digest(
    contract: &MaterializedProjectionContract,
    source_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_certified_source_receipt_v1")
            .field_shape(WorthQueryEvidenceTag::new("source"), source_digest)
            .field_shape(
                WorthQueryEvidenceTag::new("query"),
                contract
                    .query_digest()
                    .unwrap_or("no-query-owned-source-receipt"),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis"),
                contract
                    .basis_digest()
                    .unwrap_or("no-query-owned-source-basis-receipt"),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("result"),
                contract
                    .result_digest()
                    .unwrap_or("no-query-owned-source-result-receipt"),
            ),
    )?)
}

pub(crate) fn compose_negative_dx_boundary_digest(
    public_boundary_audit_digest: &str,
    compile_fail_boundary_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_negative_dx_boundary_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("public_boundary"),
                public_boundary_audit_digest,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("compile_fail_boundary"),
                compile_fail_boundary_digest,
            ),
    )?)
}

pub(crate) fn compose_failure_digest_bundle(
    denied_masked_field_failure_digest: &str,
    source_mismatch_failure_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "projection_consumption_failure_digest_bundle_v1",
        "failure",
        [
            denied_masked_field_failure_digest,
            source_mismatch_failure_digest,
        ],
    )?)
}

pub(crate) fn compose_target_dx_digest() -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "projection_consumption_target_dx_v1",
        "capability",
        [
            "common_path_read_backed_consumption",
            "common_path_effect_backed_consumption",
            "support_discovery_before_consumption",
            "typed_denial_and_deferred_handling",
            "receipt_first_inspection_and_envelope_derivation",
            "retained_live_ordinary_projection_consumption",
        ],
    )?)
}

fn compose_certification_bundle_output_entry_digest(
    name: &'static str,
    value: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_certification_bundle_output_entry_v1")
            .field_shape(WorthQueryEvidenceTag::new("name"), name)
            .field_shape(WorthQueryEvidenceTag::new("value"), value),
    )?)
}

pub(crate) fn compose_certification_bundle_digest(
    row_digests: impl IntoIterator<Item = impl AsRef<str>>,
    outputs: impl IntoIterator<Item = (&'static str, impl AsRef<str>)>,
) -> Result<String, IdentityDerivationDenial> {
    let row_entries = row_digests
        .into_iter()
        .map(|digest| digest.as_ref().to_string())
        .collect::<Vec<_>>();
    let output_entries = outputs
        .into_iter()
        .map(|(name, value)| compose_certification_bundle_output_entry_digest(name, value.as_ref()))
        .collect::<Vec<_>>();
    Ok(seal(
        certification_scope_encoder("projection_consumption_certification_bundle_v1")
            .field_value_sequence(WorthQueryEvidenceTag::new("row"), row_entries)
            .field_value_sequence(WorthQueryEvidenceTag::new("output"), output_entries),
    )?)
}

pub(crate) fn compose_support_traceability_row_digest(
    rule: &str,
    hostile: &str,
    lane: &str,
    proof: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_support_traceability_row_v1")
            .field_shape(WorthQueryEvidenceTag::new("rule"), rule)
            .field_shape(WorthQueryEvidenceTag::new("hostile"), hostile)
            .field_shape(WorthQueryEvidenceTag::new("lane"), lane)
            .field_shape(WorthQueryEvidenceTag::new("proof"), proof),
    )?)
}

pub(crate) fn compose_support_traceability_digest(
    row_digests: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "projection_consumption_support_traceability_v1",
        "row",
        row_digests,
    )?)
}

pub(crate) fn compose_forbidden_fallback_audit_digest(
    row_digests: impl IntoIterator<Item = impl AsRef<str>>,
    total_occurrence_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    let row_entries = row_digests
        .into_iter()
        .map(|digest| digest.as_ref().to_string())
        .collect::<Vec<_>>();
    Ok(seal(
        certification_scope_encoder("projection_consumption_forbidden_fallback_audit_v1")
            .field_value_sequence(WorthQueryEvidenceTag::new("row"), row_entries)
            .field_usize(
                WorthQueryEvidenceTag::new("total_occurrence_count"),
                total_occurrence_count,
            ),
    )?)
}

pub(crate) fn compose_phase_progression_digest(
    proof_shape_digest: &str,
    progression: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_phase_progression_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("proof_shape"),
                proof_shape_digest,
            )
            .field_shape(WorthQueryEvidenceTag::new("progression"), progression),
    )?)
}

pub(crate) fn compose_certified_surface_representative_digest(
    surface: &str,
    source_family: ProjectionSourceFamily,
    source_identity: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_certified_surface_v1")
            .field_shape(WorthQueryEvidenceTag::new("surface"), surface)
            .field_shape(WorthQueryEvidenceTag::new("family"), source_family.as_str())
            .field_shape(WorthQueryEvidenceTag::new("identity"), source_identity),
    )?)
}

pub(crate) fn compose_support_matrix_support_width_digest(
    support_digests: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_sequence_digest(
        "projection_consumption_support_matrix_support_width_v1",
        "support_digest",
        support_digests,
    )?)
}
