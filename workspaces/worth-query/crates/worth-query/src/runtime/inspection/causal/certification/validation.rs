use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::materialization::QueryCausalInspectionArtifact;
use super::artifacts::{
    CausalInspectionBoundaryAudit, CausalInspectionCertificationBundle,
    CausalInspectionCertificationScope, CausalInspectionScaleCounterSnapshot,
};
use super::error::{CausalInspectionCertificationError, CausalInspectionCertificationErrorKind};

pub fn certify_causal_inspection_runtime_path(
    scope: CausalInspectionCertificationScope,
) -> Result<CausalInspectionCertificationBundle, IdentityDerivationDenial> {
    let parts = scope.into_bundle_parts();
    let certification_bundle_identity = WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_certification_bundle_v1",
    )
    .field_value(
        WorthQueryEvidenceTag::new("scope"),
        &parts.certification_scope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("performance"),
        &parts.performance_certification_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("readmission"),
        &parts.bridge_readmission_proof_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("scale_slope"),
        &parts.scale_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("anchor_slope"),
        &parts.anchor_derivation_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("reference_slope"),
        &parts.reference_resolution_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("admission_slope"),
        &parts.admission_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("bridge_slope"),
        &parts.bridge_envelope_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("materialization_slope"),
        &parts.materialization_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("serialization"),
        &parts.artifact_serialization_slope_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("boundary"),
        &parts.boundary_audit_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("representatives"),
        &parts.representative_matrix_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("proof_shape"),
        &parts.proof_shape_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("phase_progression"),
        &parts.phase_progression_digest,
    )
    .field_value(
        WorthQueryEvidenceTag::new("witness_authority"),
        &parts.witness_authority_digest,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("row_count"),
        parts.certification_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("hostile_count"),
        parts.hostile_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("representative_count"),
        parts.representative_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("scale_count"),
        parts.scale_fixture_row_count,
    )
    .seal()?;
    Ok(CausalInspectionCertificationBundle::from_parts(
        certification_bundle_identity,
        parts.certification_scope_digest,
        parts.performance_certification_digest,
        parts.bridge_readmission_proof_digest,
        parts.scale_slope_digest,
        parts.anchor_derivation_slope_digest,
        parts.reference_resolution_slope_digest,
        parts.admission_slope_digest,
        parts.bridge_envelope_slope_digest,
        parts.materialization_slope_digest,
        parts.artifact_serialization_slope_digest,
        parts.boundary_audit_digest,
        parts.representative_matrix_digest,
        parts.proof_shape_digest,
        parts.phase_progression_digest,
        parts.witness_authority_digest,
        parts.certification_row_count,
        parts.hostile_row_count,
        parts.representative_row_count,
        parts.scale_fixture_row_count,
    ))
}

pub(super) fn validate_required_artifact_lanes(
    changed_artifact: &QueryCausalInspectionArtifact,
    denied_artifact: &QueryCausalInspectionArtifact,
) -> Result<(), CausalInspectionCertificationError> {
    if !changed_artifact.is_admitted() || !denied_artifact.is_denied() {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::MissingRequiredHostileLane,
            "causal certification requires admitted changed and denied hostile rows",
            &[
                format!("changed-kind:{}", changed_artifact.kind().as_str()),
                format!("denied-kind:{}", denied_artifact.kind().as_str()),
            ],
        ));
    }
    Ok(())
}

pub(super) fn validate_redaction_identity(
    full_artifact: &QueryCausalInspectionArtifact,
    redacted_artifact: &QueryCausalInspectionArtifact,
) -> Result<(), CausalInspectionCertificationError> {
    let same_identity = full_artifact.causal_identity() == redacted_artifact.causal_identity();
    let changed_detail = full_artifact.artifact_identity() != redacted_artifact.artifact_identity();
    if !same_identity || !changed_detail {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::RedactionIdentityDrift,
            "redaction rows must change materialized detail without changing causal identity",
            &[
                format!("same-identity:{same_identity}"),
                format!("changed-detail:{changed_detail}"),
            ],
        ));
    }
    Ok(())
}

pub(super) fn validate_boundary_audit(
    boundary_audit: &CausalInspectionBoundaryAudit,
) -> Result<(), CausalInspectionCertificationError> {
    if !boundary_audit.ordinary_path_uses_query_artifact()
        || !boundary_audit.direct_lower_runtime_stitching_absent()
    {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::PublicBoundaryBypass,
            "ordinary explanation path must certify Query artifact consumption",
            &[format!("audit:{}", boundary_audit.audit_digest())],
        ));
    }
    Ok(())
}

pub(super) fn validate_missing_evidence_digest(
    missing_evidence_failure_digest: &str,
) -> Result<(), CausalInspectionCertificationError> {
    if missing_evidence_failure_digest.is_empty() {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::MissingRequiredHostileLane,
            "missing-evidence row requires a typed denial failure digest",
            &["missing-evidence:none".to_string()],
        ));
    }
    Ok(())
}

pub(super) fn validate_scale_slope(
    small: &CausalInspectionScaleCounterSnapshot,
    medium: &CausalInspectionScaleCounterSnapshot,
    large: &CausalInspectionScaleCounterSnapshot,
) -> Result<(), CausalInspectionCertificationError> {
    let snapshots = [small, medium, large];
    let stable_slope = snapshots.iter().all(|snapshot| {
        snapshot.anchor_derivation_slope_counter() == 1
            && snapshot.reference_resolution_slope_counter() == 1
            && snapshot.admission_slope_counter() == 1
            && snapshot.bridge_envelope_slope_counter() == 1
            && snapshot.materialization_slope_counter() == 1
            && snapshot.artifact_serialization_slope_counter() == 1
            && snapshot.bridge_unindexed_scan_count() == 0
            && snapshot.bridge_readmission_proof_digest().is_some()
    });
    if !stable_slope {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::ScaleSlopeDrift,
            "scale rows must prove fixed inspection materialization slope counters",
            &snapshots
                .iter()
                .map(|snapshot| {
                    format!(
                        "{}:{}",
                        snapshot.fixture_size().as_str(),
                        snapshot.snapshot_digest()
                    )
                })
                .collect::<Vec<_>>(),
        ));
    }
    Ok(())
}
