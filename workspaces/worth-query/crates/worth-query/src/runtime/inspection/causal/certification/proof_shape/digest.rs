use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

#[cfg(test)]
use super::CausalInspectionProofShapeCertification;

pub(super) fn derive_phase_progression_digest(
    inspected_artifact_digest: &str,
    representative_matrix_digest: &str,
    boundary_audit_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_phase_progression_v1",
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("phase"),
        [
            "artifact-materialized",
            "representative-matrix-certified",
            "boundary-audit-certified",
        ],
    )
    .field_value(
        WorthQueryEvidenceTag::new("artifact"),
        inspected_artifact_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("matrix"),
        representative_matrix_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        boundary_audit_digest,
    )
    .seal()?
    .as_str()
    .to_string())
}

pub(super) fn derive_witness_authority_digest(
    inspected_artifact_digest: &str,
    representative_matrix_digest: &str,
    boundary_audit_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_witness_authority_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("artifact_authority"),
        "query-causal-inspection-artifact",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("matrix_authority"),
        "causal-inspection-representative-matrix",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("boundary_authority"),
        "causal-inspection-boundary-audit",
    )
    .field_value(
        WorthQueryEvidenceTag::new("artifact"),
        inspected_artifact_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("matrix"),
        representative_matrix_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        boundary_audit_digest,
    )
    .seal()?
    .as_str()
    .to_string())
}

pub(super) fn canonical_proof_shape_digest(
    inspected_artifact_digest: &str,
    representative_matrix_digest: &str,
    boundary_audit_digest: &str,
    phase_progression_digest: &str,
    witness_authority_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_proof_shape_certification_v1",
    )
    .field_bool(WorthQueryEvidenceTag::new("phase_skipping_rejected"), true)
    .field_bool(
        WorthQueryEvidenceTag::new("raw_collection_substitution_rejected"),
        true,
    )
    .field_bool(
        WorthQueryEvidenceTag::new("stale_proof_reuse_rejected"),
        true,
    )
    .field_bool(
        WorthQueryEvidenceTag::new("worthd_authority_witness_rejected"),
        true,
    )
    .field_value(
        WorthQueryEvidenceTag::new("artifact"),
        inspected_artifact_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("matrix"),
        representative_matrix_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        boundary_audit_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("phase_progression"),
        phase_progression_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("witness_authority"),
        witness_authority_digest,
    )
    .seal()?
    .as_str()
    .to_string())
}

#[cfg(test)]
pub(super) fn stale_test_proof_shape_digest(
    inspected_artifact_digest: &str,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_stale_test_proof_shape_v1",
    )
    .field_bool(WorthQueryEvidenceTag::new("stale_test_digest"), true)
    .field_value(
        WorthQueryEvidenceTag::new("artifact"),
        inspected_artifact_digest,
    )
    .seal()?
    .as_str()
    .to_string())
}

#[cfg(test)]
pub(super) fn worthd_test_proof_shape_digest(
    proof: &CausalInspectionProofShapeCertification,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_worthd_test_proof_shape_v1",
    )
    .field_bool(WorthQueryEvidenceTag::new("worthd_test"), false)
    .field_value(
        WorthQueryEvidenceTag::new("artifact"),
        proof.inspected_artifact_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("matrix"),
        proof.representative_matrix_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        proof.boundary_audit_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("phase"),
        proof.phase_progression_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("witness"),
        proof.witness_authority_digest(),
    )
    .seal()?
    .as_str()
    .to_string())
}
