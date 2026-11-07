use super::super::super::materialization::QueryCausalInspectionArtifact;
use super::super::artifacts::CausalInspectionBoundaryAudit;
use super::super::error::{
    CausalInspectionCertificationError, CausalInspectionCertificationErrorKind,
};
use super::super::matrix::CausalInspectionRepresentativeMatrix;
use super::digest::canonical_proof_shape_digest;
use super::CausalInspectionProofShapeCertification;

pub(in crate::runtime::inspection::causal::certification) fn validate_proof_shape(
    proof_shape: &CausalInspectionProofShapeCertification,
    changed_artifact: &QueryCausalInspectionArtifact,
    representative_matrix: &CausalInspectionRepresentativeMatrix,
    boundary_audit: &CausalInspectionBoundaryAudit,
) -> Result<(), CausalInspectionCertificationError> {
    let posture_complete = proof_shape.phase_skipping_rejected()
        && proof_shape.raw_collection_substitution_rejected()
        && proof_shape.stale_proof_reuse_rejected()
        && proof_shape.worthd_authority_witness_rejected();
    let binds_artifact =
        proof_shape.inspected_artifact_digest() == changed_artifact.artifact_identity().as_str();
    let binds_matrix =
        proof_shape.representative_matrix_digest() == representative_matrix.matrix_digest();
    let binds_boundary = proof_shape.boundary_audit_digest() == boundary_audit.audit_digest();
    let canonical_shape = proof_shape.proof_shape_digest()
        == canonical_proof_shape_digest(
            proof_shape.inspected_artifact_digest(),
            proof_shape.representative_matrix_digest(),
            proof_shape.boundary_audit_digest(),
            proof_shape.phase_progression_digest(),
            proof_shape.witness_authority_digest(),
        )?;
    if posture_complete && binds_artifact && binds_matrix && binds_boundary && canonical_shape {
        return Ok(());
    }
    Err(CausalInspectionCertificationError::new(
        CausalInspectionCertificationErrorKind::ProofShapeBypass,
        "proof-shape certification must reject phase skipping, raw substitution, stale reuse, and Worthd witnesses for this runtime path",
        &[
            format!("posture-complete:{posture_complete}"),
            format!("binds-artifact:{binds_artifact}"),
            format!("binds-matrix:{binds_matrix}"),
            format!("binds-boundary:{binds_boundary}"),
            format!("canonical-shape:{canonical_shape}"),
        ],
    ))
}
