use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::materialization::QueryCausalInspectionArtifact;
use super::artifacts::{
    CausalInspectionBoundaryAudit, CausalInspectionCertificationLane,
    CausalInspectionCertificationScope, CausalInspectionPerformanceCertificationBundle,
    CausalInspectionScaleCounterSnapshot,
};
use super::error::CausalInspectionCertificationError;
use super::matrix::CausalInspectionRepresentativeMatrix;
use super::proof_shape::{validate_proof_shape, CausalInspectionProofShapeCertification};
use super::validation::{
    validate_boundary_audit, validate_missing_evidence_digest, validate_redaction_identity,
    validate_required_artifact_lanes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CausalInspectionHostileRow {
    pub(super) row_digest: String,
}

struct CausalInspectionCertificationHostileRows {
    changed_row: CausalInspectionHostileRow,
    redaction_row: CausalInspectionHostileRow,
    denied_row: CausalInspectionHostileRow,
    missing_evidence_row: CausalInspectionHostileRow,
    public_boundary_row: CausalInspectionHostileRow,
    representative_matrix_row: CausalInspectionHostileRow,
    scale_row: CausalInspectionHostileRow,
    proof_shape_row: CausalInspectionHostileRow,
    bridge_readmission_row: CausalInspectionHostileRow,
    artifact_serialization_row: CausalInspectionHostileRow,
}

struct CausalInspectionCertificationRowCounts {
    certification_row_count: usize,
    hostile_row_count: usize,
    scale_fixture_row_count: usize,
}

impl CausalInspectionHostileRow {
    fn from_artifact(
        lane: CausalInspectionCertificationLane,
        artifact: &QueryCausalInspectionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            lane,
            artifact.artifact_identity().evidence_identity(),
        )?)
    }

    fn new(
        lane: CausalInspectionCertificationLane,
        evidence_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_hostile_row_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("lane"), lane.as_str())
        .field_evidence_identity(WorthQueryEvidenceTag::new("evidence"), evidence_identity)
        .seal()?;
        Ok(Self {
            row_digest: row_identity.as_str().to_string(),
        })
    }

    fn from_terminal_digest(
        lane: CausalInspectionCertificationLane,
        evidence_digest: &str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_hostile_terminal_digest_row_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("lane"), lane.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("terminal_evidence_digest"),
            evidence_digest,
        )
        .seal()?;
        Ok(Self {
            row_digest: row_identity.as_str().to_string(),
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_causal_inspection_certification_scope(
    changed_artifact: &QueryCausalInspectionArtifact,
    full_artifact: &QueryCausalInspectionArtifact,
    redacted_artifact: &QueryCausalInspectionArtifact,
    denied_artifact: &QueryCausalInspectionArtifact,
    missing_evidence_failure_digest: &str,
    boundary_audit: CausalInspectionBoundaryAudit,
    representative_matrix: CausalInspectionRepresentativeMatrix,
    proof_shape: CausalInspectionProofShapeCertification,
    small: CausalInspectionScaleCounterSnapshot,
    medium: CausalInspectionScaleCounterSnapshot,
    large: CausalInspectionScaleCounterSnapshot,
) -> Result<CausalInspectionCertificationScope, CausalInspectionCertificationError> {
    validate_certification_scope_inputs(
        changed_artifact,
        full_artifact,
        redacted_artifact,
        denied_artifact,
        missing_evidence_failure_digest,
        &boundary_audit,
        &proof_shape,
        &representative_matrix,
    )?;

    let performance_certification =
        CausalInspectionPerformanceCertificationBundle::from_snapshots(&small, &medium, &large)?;
    let bridge_readmission_proof_digest = performance_certification
        .bridge_readmission_proof_digest()
        .to_string();
    let scale_slope_digest = performance_certification.scale_slope_digest().to_string();
    let anchor_derivation_slope_digest = performance_certification
        .anchor_derivation_slope_digest()
        .to_string();
    let reference_resolution_slope_digest = performance_certification
        .reference_resolution_slope_digest()
        .to_string();
    let admission_slope_digest = performance_certification
        .admission_slope_digest()
        .to_string();
    let bridge_envelope_slope_digest = performance_certification
        .bridge_envelope_slope_digest()
        .to_string();
    let materialization_slope_digest = performance_certification
        .materialization_slope_digest()
        .to_string();
    let artifact_serialization_slope_digest = performance_certification
        .artifact_serialization_slope_digest()
        .to_string();
    let hostile_rows = build_certification_hostile_rows(
        changed_artifact,
        redacted_artifact,
        denied_artifact,
        missing_evidence_failure_digest,
        &boundary_audit,
        &representative_matrix,
        &performance_certification,
        &proof_shape,
    )?;
    let row_counts = certification_row_counts(&representative_matrix);
    let scope_digest = build_certification_scope_digest(
        &hostile_rows,
        &representative_matrix,
        &performance_certification,
        &proof_shape,
        &row_counts,
    )?;
    Ok(CausalInspectionCertificationScope::from_parts(
        boundary_audit.audit_digest(),
        representative_matrix.matrix_digest(),
        performance_certification,
        &bridge_readmission_proof_digest,
        &scale_slope_digest,
        &anchor_derivation_slope_digest,
        &reference_resolution_slope_digest,
        &admission_slope_digest,
        &bridge_envelope_slope_digest,
        &materialization_slope_digest,
        &artifact_serialization_slope_digest,
        proof_shape.proof_shape_digest(),
        proof_shape.phase_progression_digest(),
        proof_shape.witness_authority_digest(),
        row_counts.certification_row_count,
        row_counts.hostile_row_count,
        representative_matrix.representative_count(),
        row_counts.scale_fixture_row_count,
        scope_digest,
    ))
}

fn validate_certification_scope_inputs(
    changed_artifact: &QueryCausalInspectionArtifact,
    full_artifact: &QueryCausalInspectionArtifact,
    redacted_artifact: &QueryCausalInspectionArtifact,
    denied_artifact: &QueryCausalInspectionArtifact,
    missing_evidence_failure_digest: &str,
    boundary_audit: &CausalInspectionBoundaryAudit,
    proof_shape: &CausalInspectionProofShapeCertification,
    representative_matrix: &CausalInspectionRepresentativeMatrix,
) -> Result<(), CausalInspectionCertificationError> {
    validate_required_artifact_lanes(changed_artifact, denied_artifact)?;
    validate_redaction_identity(full_artifact, redacted_artifact)?;
    validate_boundary_audit(boundary_audit)?;
    validate_missing_evidence_digest(missing_evidence_failure_digest)?;
    validate_proof_shape(
        proof_shape,
        changed_artifact,
        representative_matrix,
        boundary_audit,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_certification_hostile_rows(
    changed_artifact: &QueryCausalInspectionArtifact,
    redacted_artifact: &QueryCausalInspectionArtifact,
    denied_artifact: &QueryCausalInspectionArtifact,
    missing_evidence_failure_digest: &str,
    boundary_audit: &CausalInspectionBoundaryAudit,
    representative_matrix: &CausalInspectionRepresentativeMatrix,
    performance_certification: &CausalInspectionPerformanceCertificationBundle,
    proof_shape: &CausalInspectionProofShapeCertification,
) -> Result<CausalInspectionCertificationHostileRows, IdentityDerivationDenial> {
    let changed_row = CausalInspectionHostileRow::from_artifact(
        CausalInspectionCertificationLane::ChangedResult,
        changed_artifact,
    )?;
    let redaction_row = CausalInspectionHostileRow::from_artifact(
        CausalInspectionCertificationLane::PolicyRedacted,
        redacted_artifact,
    )?;
    let denied_row = CausalInspectionHostileRow::from_artifact(
        CausalInspectionCertificationLane::DeniedResult,
        denied_artifact,
    )?;
    let missing_evidence_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::MissingEvidence,
        missing_evidence_failure_digest,
    )?;
    let public_boundary_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::PublicBoundary,
        boundary_audit.audit_digest(),
    )?;
    let representative_matrix_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::RepresentativeMatrix,
        representative_matrix.matrix_digest(),
    )?;
    let scale_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::ScaleMaterialization,
        performance_certification.performance_certification_digest(),
    )?;
    let proof_shape_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::ProofShape,
        proof_shape.proof_shape_digest(),
    )?;
    let bridge_readmission_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::BridgeReadmission,
        performance_certification.bridge_readmission_proof_digest(),
    )?;
    let artifact_serialization_row = CausalInspectionHostileRow::from_terminal_digest(
        CausalInspectionCertificationLane::ArtifactSerialization,
        performance_certification.artifact_serialization_slope_digest(),
    )?;
    Ok(CausalInspectionCertificationHostileRows {
        changed_row,
        redaction_row,
        denied_row,
        missing_evidence_row,
        public_boundary_row,
        representative_matrix_row,
        scale_row,
        proof_shape_row,
        bridge_readmission_row,
        artifact_serialization_row,
    })
}

fn certification_row_counts(
    representative_matrix: &CausalInspectionRepresentativeMatrix,
) -> CausalInspectionCertificationRowCounts {
    CausalInspectionCertificationRowCounts {
        certification_row_count: 10 + representative_matrix.representative_count(),
        hostile_row_count: 10,
        scale_fixture_row_count: 3,
    }
}

fn build_certification_scope_digest(
    hostile_rows: &CausalInspectionCertificationHostileRows,
    representative_matrix: &CausalInspectionRepresentativeMatrix,
    performance_certification: &CausalInspectionPerformanceCertificationBundle,
    proof_shape: &CausalInspectionProofShapeCertification,
    row_counts: &CausalInspectionCertificationRowCounts,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_certification_scope_v1",
    )
    .field_value(
        WorthQueryEvidenceTag::new("changed"),
        &hostile_rows.changed_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("redaction"),
        &hostile_rows.redaction_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("denied"),
        &hostile_rows.denied_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("missing"),
        &hostile_rows.missing_evidence_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        &hostile_rows.public_boundary_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("representatives"),
        &hostile_rows.representative_matrix_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("scale"),
        &hostile_rows.scale_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("proof_shape"),
        &hostile_rows.proof_shape_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("bridge_readmission"),
        &hostile_rows.bridge_readmission_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("artifact_serialization"),
        &hostile_rows.artifact_serialization_row.row_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("matrix"),
        representative_matrix.matrix_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("scale_slope"),
        performance_certification.scale_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("anchor_slope"),
        performance_certification.anchor_derivation_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("reference_slope"),
        performance_certification.reference_resolution_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("admission_slope"),
        performance_certification.admission_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("bridge_envelope_slope"),
        performance_certification.bridge_envelope_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("materialization_slope"),
        performance_certification.materialization_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("serialization_slope"),
        performance_certification.artifact_serialization_slope_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("proof_shape_digest"),
        proof_shape.proof_shape_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("phase_progression"),
        proof_shape.phase_progression_digest(),
    )
    .field_value(
        WorthQueryEvidenceTag::new("witness_authority"),
        proof_shape.witness_authority_digest(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("row_count"),
        row_counts.certification_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("hostile_count"),
        row_counts.hostile_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("representative_count"),
        representative_matrix.representative_count(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("scale_count"),
        row_counts.scale_fixture_row_count,
    )
    .seal()?
    .as_str()
    .to_string())
}
