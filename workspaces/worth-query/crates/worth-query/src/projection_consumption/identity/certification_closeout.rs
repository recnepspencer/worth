use super::certification::compose_certification_row_digest;
use super::scope::{certification_scope_encoder, seal};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::WorthQueryEvidenceTag;

pub(crate) fn compose_closeout_support_matrix_surface_row_digest(
    inventory_digest: &str,
    matrix_digest: &str,
    traceability_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_row_digest(
        "projection_consumption_certification_row_v1",
        &[
            ("lane", "support_matrix_surface"),
            ("inventory", inventory_digest),
            ("matrix", matrix_digest),
            ("traceability", traceability_digest),
        ],
    )?)
}

pub(crate) fn compose_closeout_public_boundary_surface_row_digest(
    public_surface_digest: &str,
    negative_dx_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_row_digest(
        "projection_consumption_certification_row_v1",
        &[
            ("lane", "public_boundary_surface"),
            ("public_surface", public_surface_digest),
            ("negative_dx", negative_dx_digest),
        ],
    )?)
}

pub(crate) fn compose_closeout_proof_shape_surface_row_digest(
    proof_shape_digest: &str,
    phase_progression_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_row_digest(
        "projection_consumption_certification_row_v1",
        &[
            ("lane", "proof_shape_surface"),
            ("proof_shape", proof_shape_digest),
            ("phase_progression", phase_progression_digest),
        ],
    )?)
}

pub(crate) fn compose_closeout_forbidden_fallback_surface_row_digest(
    forbidden_fallback_digest: &str,
    total_occurrences: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(seal(
        certification_scope_encoder("projection_consumption_certification_row_v1")
            .field_shape(
                WorthQueryEvidenceTag::new("lane"),
                "forbidden_fallback_surface",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("forbidden_fallback"),
                forbidden_fallback_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("total_occurrences"),
                total_occurrences,
            ),
    )?)
}

pub(crate) fn compose_closeout_oracle_surface_row_digest(
    oracle_digest: &str,
    manifest_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_row_digest(
        "projection_consumption_certification_row_v1",
        &[
            ("lane", "oracle_surface"),
            ("oracle", oracle_digest),
            ("manifest", manifest_digest),
        ],
    )?)
}

pub(crate) fn compose_closeout_seeded_replay_surface_row_digest(
    seeded_digest: &str,
    replay_digest: &str,
    generator_classes_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(compose_certification_row_digest(
        "projection_consumption_certification_row_v1",
        &[
            ("lane", "seeded_replay_surface"),
            ("seeded", seeded_digest),
            ("replay", replay_digest),
            ("classes", generator_classes_digest),
        ],
    )?)
}
