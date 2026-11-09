use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod digest;
mod validation;

use super::super::materialization::QueryCausalInspectionArtifact;
use super::artifacts::CausalInspectionBoundaryAudit;
use super::matrix::CausalInspectionRepresentativeMatrix;
use digest::{
    canonical_proof_shape_digest, derive_phase_progression_digest, derive_witness_authority_digest,
};
pub(in crate::runtime::inspection::causal::certification) use validation::validate_proof_shape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionProofShapeCertification {
    phase_skipping_rejected: bool,
    raw_collection_substitution_rejected: bool,
    stale_proof_reuse_rejected: bool,
    worthd_authority_witness_rejected: bool,
    inspected_artifact_digest: String,
    representative_matrix_digest: String,
    boundary_audit_digest: String,
    phase_progression_digest: String,
    witness_authority_digest: String,
    proof_shape_digest: String,
}

impl CausalInspectionProofShapeCertification {
    pub fn from_runtime_path(
        changed_artifact: &QueryCausalInspectionArtifact,
        representative_matrix: &CausalInspectionRepresentativeMatrix,
        boundary_audit: &CausalInspectionBoundaryAudit,
    ) -> Result<Self, IdentityDerivationDenial> {
        let inspected_artifact_digest = changed_artifact.artifact_identity().as_str().to_string();
        let representative_matrix_digest = representative_matrix.matrix_digest().to_string();
        let boundary_audit_digest = boundary_audit.audit_digest().to_string();
        let phase_progression_digest = derive_phase_progression_digest(
            &inspected_artifact_digest,
            &representative_matrix_digest,
            &boundary_audit_digest,
        )?;
        let witness_authority_digest = derive_witness_authority_digest(
            &inspected_artifact_digest,
            &representative_matrix_digest,
            &boundary_audit_digest,
        )?;
        let proof_shape_digest = canonical_proof_shape_digest(
            &inspected_artifact_digest,
            &representative_matrix_digest,
            &boundary_audit_digest,
            &phase_progression_digest,
            &witness_authority_digest,
        )?;
        Ok(Self {
            phase_skipping_rejected: true,
            raw_collection_substitution_rejected: true,
            stale_proof_reuse_rejected: true,
            worthd_authority_witness_rejected: true,
            inspected_artifact_digest,
            representative_matrix_digest,
            boundary_audit_digest,
            phase_progression_digest,
            witness_authority_digest,
            proof_shape_digest,
        })
    }

    #[cfg(test)]
    pub(in crate::runtime) fn stale_digest_for_tests(
        changed_artifact: &QueryCausalInspectionArtifact,
        representative_matrix: &CausalInspectionRepresentativeMatrix,
        boundary_audit: &CausalInspectionBoundaryAudit,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut proof =
            Self::from_runtime_path(changed_artifact, representative_matrix, boundary_audit)?;
        proof.proof_shape_digest =
            digest::stale_test_proof_shape_digest(&proof.inspected_artifact_digest)?;
        Ok(proof)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn worthd_for_tests(
        changed_artifact: &QueryCausalInspectionArtifact,
        representative_matrix: &CausalInspectionRepresentativeMatrix,
        boundary_audit: &CausalInspectionBoundaryAudit,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut proof =
            Self::from_runtime_path(changed_artifact, representative_matrix, boundary_audit)?;
        proof.worthd_authority_witness_rejected = false;
        proof.proof_shape_digest = digest::worthd_test_proof_shape_digest(&proof)?;
        Ok(proof)
    }

    pub fn phase_skipping_rejected(&self) -> bool {
        self.phase_skipping_rejected
    }

    pub fn raw_collection_substitution_rejected(&self) -> bool {
        self.raw_collection_substitution_rejected
    }

    pub fn stale_proof_reuse_rejected(&self) -> bool {
        self.stale_proof_reuse_rejected
    }

    pub fn worthd_authority_witness_rejected(&self) -> bool {
        self.worthd_authority_witness_rejected
    }

    pub fn inspected_artifact_digest(&self) -> &str {
        &self.inspected_artifact_digest
    }

    pub fn representative_matrix_digest(&self) -> &str {
        &self.representative_matrix_digest
    }

    pub fn boundary_audit_digest(&self) -> &str {
        &self.boundary_audit_digest
    }

    pub fn phase_progression_digest(&self) -> &str {
        &self.phase_progression_digest
    }

    pub fn witness_authority_digest(&self) -> &str {
        &self.witness_authority_digest
    }

    pub fn proof_shape_digest(&self) -> &str {
        &self.proof_shape_digest
    }
}
