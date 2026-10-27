use super::*;

pub(in crate::runtime::inspection::causal) fn compose_causal_performance_snapshot_identity(
    fixture_size: CausalInspectionScaleFixtureSize,
    artifact_identity: &CausalInspectionArtifactIdentity,
    evidence_reference_width: usize,
    anchor_derivation_slope_counter: usize,
    reference_resolution_slope_counter: usize,
    admission_slope_counter: usize,
    bridge_envelope_slope_counter: usize,
    materialization_slope_counter: usize,
    artifact_serialization_slope_counter: usize,
    bridge_unindexed_scan_count: usize,
    bridge_readmission_proof: Option<&WorthQueryEvidenceIdentity>,
) -> CausalInspectionPerformanceSnapshotIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionPerformanceSnapshot)
        .field_shape(WorthQueryEvidenceTag::new("size"), fixture_size.as_str())
        .field_evidence_identity(
            causal_artifact_identity_tag(),
            artifact_identity.evidence_identity(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("evidence_width"),
            evidence_reference_width,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("anchor_slope"),
            anchor_derivation_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("reference_slope"),
            reference_resolution_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("admission_slope"),
            admission_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_envelope_slope"),
            bridge_envelope_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("materialization_slope"),
            materialization_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("serialization_slope"),
            artifact_serialization_slope_counter,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("bridge_unindexed_scan"),
            bridge_unindexed_scan_count,
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("readmission"),
            bridge_readmission_proof,
        )
        .seal()
        .into()
}

fn causal_artifact_identity_tag() -> WorthQueryEvidenceTag {
    WorthQueryEvidenceTag::new("artifact")
}

pub(in crate::runtime::inspection::causal) fn compose_causal_performance_slope_identity(
    label: &str,
    small: usize,
    medium: usize,
    large: usize,
) -> CausalInspectionPerformanceSlopeIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionPerformanceSlope)
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .field_usize(WorthQueryEvidenceTag::new("small"), small)
        .field_usize(WorthQueryEvidenceTag::new("medium"), medium)
        .field_usize(WorthQueryEvidenceTag::new("large"), large)
        .seal()
        .into()
}

pub(in crate::runtime::inspection::causal) fn compose_causal_performance_scale_slope_identity(
    anchor_derivation_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    reference_resolution_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    admission_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    bridge_envelope_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    materialization_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    artifact_serialization_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
) -> CausalInspectionPerformanceScaleSlopeIdentity {
    worth_query_evidence_identity(WorthQueryEvidenceScope::CausalInspectionPerformanceScaleSlope)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            anchor_derivation_slope_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("reference"),
            reference_resolution_slope_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission"),
            admission_slope_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_envelope"),
            bridge_envelope_slope_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("materialization"),
            materialization_slope_identity.evidence_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("serialization"),
            artifact_serialization_slope_identity.evidence_identity(),
        )
        .seal()
        .into()
}

pub(in crate::runtime::inspection::causal) fn compose_causal_performance_certification_identity(
    small_snapshot_identity: &CausalInspectionPerformanceSnapshotIdentity,
    medium_snapshot_identity: &CausalInspectionPerformanceSnapshotIdentity,
    large_snapshot_identity: &CausalInspectionPerformanceSnapshotIdentity,
    bridge_readmission_proof_identity: &WorthQueryEvidenceIdentity,
    scale_slope_identity: &CausalInspectionPerformanceScaleSlopeIdentity,
    anchor_derivation_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    reference_resolution_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    admission_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    bridge_envelope_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    materialization_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    artifact_serialization_slope_identity: &CausalInspectionPerformanceSlopeIdentity,
    scale_slope_digest_part_count: usize,
) -> CausalInspectionPerformanceCertificationIdentity {
    worth_query_evidence_identity(
        WorthQueryEvidenceScope::CausalInspectionPerformanceCertificationBundle,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("small"),
        small_snapshot_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("medium"),
        medium_snapshot_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("large"),
        large_snapshot_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("readmission"),
        bridge_readmission_proof_identity,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("scale_slope"),
        scale_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("anchor"),
        anchor_derivation_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("reference"),
        reference_resolution_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("admission"),
        admission_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("bridge_envelope"),
        bridge_envelope_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("materialization"),
        materialization_slope_identity.evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("serialization"),
        artifact_serialization_slope_identity.evidence_identity(),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("parts"),
        scale_slope_digest_part_count,
    )
    .seal()
    .into()
}
