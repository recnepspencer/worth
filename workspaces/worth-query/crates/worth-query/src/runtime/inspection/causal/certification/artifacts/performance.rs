use super::super::super::materialization::QueryCausalInspectionArtifact;
use super::super::error::CausalInspectionCertificationError;
use super::super::validation::validate_scale_slope;
use super::lane::CausalInspectionScaleFixtureSize;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::inspection::causal::identity::{
    compose_causal_performance_certification_identity,
    compose_causal_performance_scale_slope_identity, compose_causal_performance_slope_identity,
    compose_causal_performance_snapshot_identity, CausalInspectionArtifactIdentity,
    CausalInspectionPerformanceSlopeIdentity, CausalInspectionPerformanceSnapshotIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionScaleCounterSnapshot {
    fixture_size: CausalInspectionScaleFixtureSize,
    artifact_identity: CausalInspectionArtifactIdentity,
    evidence_reference_width: usize,
    anchor_derivation_slope_counter: usize,
    reference_resolution_slope_counter: usize,
    admission_slope_counter: usize,
    bridge_envelope_slope_counter: usize,
    materialization_slope_counter: usize,
    artifact_serialization_slope_counter: usize,
    bridge_unindexed_scan_count: usize,
    bridge_readmission_proof_identity: Option<WorthQueryEvidenceIdentity>,
    snapshot_identity: CausalInspectionPerformanceSnapshotIdentity,
}

impl CausalInspectionScaleCounterSnapshot {
    pub fn from_artifact(
        fixture_size: CausalInspectionScaleFixtureSize,
        artifact: &QueryCausalInspectionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let performance = artifact.performance();
        let artifact_identity = artifact.artifact_identity().clone();
        let evidence_reference_width = performance.bridge_binding_count();
        let anchor_derivation_slope_counter = performance.anchor_derivation_count();
        let reference_resolution_slope_counter = performance.evidence_reference_resolution_count();
        let admission_slope_counter = performance.admission_count();
        let bridge_envelope_slope_counter = performance.bridge_envelope_assembly_count();
        let materialization_slope_counter = performance.materialization_count();
        let artifact_serialization_slope_counter = performance.artifact_serialization_count();
        let bridge_unindexed_scan_count = performance.bridge_unindexed_scan_count();
        let bridge_readmission_proof_identity =
            artifact.bridge_readmission_proof_identity().cloned();
        let snapshot_identity = snapshot_identity(SnapshotDigestParts {
            fixture_size,
            artifact_identity: &artifact_identity,
            evidence_reference_width,
            anchor_derivation_slope_counter,
            reference_resolution_slope_counter,
            admission_slope_counter,
            bridge_envelope_slope_counter,
            materialization_slope_counter,
            artifact_serialization_slope_counter,
            bridge_unindexed_scan_count,
            bridge_readmission_proof_identity: bridge_readmission_proof_identity.as_ref(),
        })?;
        Ok(Self {
            fixture_size,
            artifact_identity,
            evidence_reference_width,
            anchor_derivation_slope_counter,
            reference_resolution_slope_counter,
            admission_slope_counter,
            bridge_envelope_slope_counter,
            materialization_slope_counter,
            artifact_serialization_slope_counter,
            bridge_unindexed_scan_count,
            bridge_readmission_proof_identity,
            snapshot_identity,
        })
    }

    pub fn fixture_size(&self) -> CausalInspectionScaleFixtureSize {
        self.fixture_size
    }

    pub fn artifact_digest(&self) -> &str {
        self.artifact_identity.as_str()
    }

    pub fn evidence_reference_width(&self) -> usize {
        self.evidence_reference_width
    }

    pub fn anchor_derivation_slope_counter(&self) -> usize {
        self.anchor_derivation_slope_counter
    }

    pub fn reference_resolution_slope_counter(&self) -> usize {
        self.reference_resolution_slope_counter
    }

    pub fn admission_slope_counter(&self) -> usize {
        self.admission_slope_counter
    }

    pub fn bridge_envelope_slope_counter(&self) -> usize {
        self.bridge_envelope_slope_counter
    }

    pub fn materialization_slope_counter(&self) -> usize {
        self.materialization_slope_counter
    }

    pub fn artifact_serialization_slope_counter(&self) -> usize {
        self.artifact_serialization_slope_counter
    }

    pub fn bridge_unindexed_scan_count(&self) -> usize {
        self.bridge_unindexed_scan_count
    }

    pub fn bridge_readmission_proof_digest(&self) -> Option<&str> {
        self.bridge_readmission_proof_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn snapshot_digest(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    #[cfg(test)]
    pub(in crate::runtime) fn with_bridge_envelope_slope_for_tests(
        mut self,
        bridge_envelope_slope_counter: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        self.bridge_envelope_slope_counter = bridge_envelope_slope_counter;
        self.snapshot_identity = snapshot_identity(SnapshotDigestParts {
            fixture_size: self.fixture_size,
            artifact_identity: &self.artifact_identity,
            evidence_reference_width: self.evidence_reference_width,
            anchor_derivation_slope_counter: self.anchor_derivation_slope_counter,
            reference_resolution_slope_counter: self.reference_resolution_slope_counter,
            admission_slope_counter: self.admission_slope_counter,
            bridge_envelope_slope_counter,
            materialization_slope_counter: self.materialization_slope_counter,
            artifact_serialization_slope_counter: self.artifact_serialization_slope_counter,
            bridge_unindexed_scan_count: self.bridge_unindexed_scan_count,
            bridge_readmission_proof_identity: self.bridge_readmission_proof_identity.as_ref(),
        })?;
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionPerformanceCertificationBundle {
    small_snapshot_digest: String,
    medium_snapshot_digest: String,
    large_snapshot_digest: String,
    bridge_readmission_proof_digest: String,
    scale_slope_digest: String,
    anchor_derivation_slope_digest: String,
    reference_resolution_slope_digest: String,
    admission_slope_digest: String,
    bridge_envelope_slope_digest: String,
    materialization_slope_digest: String,
    artifact_serialization_slope_digest: String,
    scale_slope_digest_part_count: usize,
    performance_certification_digest: String,
}

impl CausalInspectionPerformanceCertificationBundle {
    pub(in crate::runtime::inspection::causal::certification) fn from_snapshots(
        small: &CausalInspectionScaleCounterSnapshot,
        medium: &CausalInspectionScaleCounterSnapshot,
        large: &CausalInspectionScaleCounterSnapshot,
    ) -> Result<Self, CausalInspectionCertificationError> {
        validate_scale_slope(small, medium, large)?;
        let bridge_readmission_proof_identity = small
            .bridge_readmission_proof_identity
            .as_ref()
            .expect("validated scale snapshots should carry readmission proof");
        let anchor_derivation_slope_identity = slope_identity(
            "causal_anchor_derivation_slope_digest_v1",
            small.anchor_derivation_slope_counter(),
            medium.anchor_derivation_slope_counter(),
            large.anchor_derivation_slope_counter(),
        );
        let reference_resolution_slope_identity = slope_identity(
            "causal_reference_resolution_slope_digest_v1",
            small.reference_resolution_slope_counter(),
            medium.reference_resolution_slope_counter(),
            large.reference_resolution_slope_counter(),
        );
        let admission_slope_identity = slope_identity(
            "causal_admission_slope_digest_v1",
            small.admission_slope_counter(),
            medium.admission_slope_counter(),
            large.admission_slope_counter(),
        );
        let bridge_envelope_slope_identity = slope_identity(
            "causal_bridge_envelope_slope_digest_v1",
            small.bridge_envelope_slope_counter(),
            medium.bridge_envelope_slope_counter(),
            large.bridge_envelope_slope_counter(),
        );
        let materialization_slope_identity = slope_identity(
            "causal_materialization_slope_digest_v1",
            small.materialization_slope_counter(),
            medium.materialization_slope_counter(),
            large.materialization_slope_counter(),
        );
        let artifact_serialization_slope_identity = compose_causal_performance_slope_identity(
            "causal_artifact_serialization_slope_digest_v1",
            small.artifact_serialization_slope_counter(),
            medium.artifact_serialization_slope_counter(),
            large.artifact_serialization_slope_counter(),
        );
        let scale_slope_identity = compose_causal_performance_scale_slope_identity(
            &anchor_derivation_slope_identity,
            &reference_resolution_slope_identity,
            &admission_slope_identity,
            &bridge_envelope_slope_identity,
            &materialization_slope_identity,
            &artifact_serialization_slope_identity,
        );
        let scale_slope_digest_part_count = 3;
        let performance_certification_identity = compose_causal_performance_certification_identity(
            &small.snapshot_identity,
            &medium.snapshot_identity,
            &large.snapshot_identity,
            bridge_readmission_proof_identity,
            &scale_slope_identity,
            &anchor_derivation_slope_identity,
            &reference_resolution_slope_identity,
            &admission_slope_identity,
            &bridge_envelope_slope_identity,
            &materialization_slope_identity,
            &artifact_serialization_slope_identity,
            scale_slope_digest_part_count,
        );
        Ok(Self {
            small_snapshot_digest: small.snapshot_digest().to_string(),
            medium_snapshot_digest: medium.snapshot_digest().to_string(),
            large_snapshot_digest: large.snapshot_digest().to_string(),
            bridge_readmission_proof_digest: bridge_readmission_proof_identity.as_str().to_string(),
            scale_slope_digest: scale_slope_identity.as_str().to_string(),
            anchor_derivation_slope_digest: anchor_derivation_slope_identity.as_str().to_string(),
            reference_resolution_slope_digest: reference_resolution_slope_identity
                .as_str()
                .to_string(),
            admission_slope_digest: admission_slope_identity.as_str().to_string(),
            bridge_envelope_slope_digest: bridge_envelope_slope_identity.as_str().to_string(),
            materialization_slope_digest: materialization_slope_identity.as_str().to_string(),
            artifact_serialization_slope_digest: artifact_serialization_slope_identity
                .as_str()
                .to_string(),
            scale_slope_digest_part_count,
            performance_certification_digest: performance_certification_identity
                .as_str()
                .to_string(),
        })
    }

    pub fn small_snapshot_digest(&self) -> &str {
        &self.small_snapshot_digest
    }

    pub fn medium_snapshot_digest(&self) -> &str {
        &self.medium_snapshot_digest
    }

    pub fn large_snapshot_digest(&self) -> &str {
        &self.large_snapshot_digest
    }

    pub fn bridge_readmission_proof_digest(&self) -> &str {
        &self.bridge_readmission_proof_digest
    }

    pub fn scale_slope_digest(&self) -> &str {
        &self.scale_slope_digest
    }

    pub fn anchor_derivation_slope_digest(&self) -> &str {
        &self.anchor_derivation_slope_digest
    }

    pub fn reference_resolution_slope_digest(&self) -> &str {
        &self.reference_resolution_slope_digest
    }

    pub fn admission_slope_digest(&self) -> &str {
        &self.admission_slope_digest
    }

    pub fn bridge_envelope_slope_digest(&self) -> &str {
        &self.bridge_envelope_slope_digest
    }

    pub fn materialization_slope_digest(&self) -> &str {
        &self.materialization_slope_digest
    }

    pub fn artifact_serialization_slope_digest(&self) -> &str {
        &self.artifact_serialization_slope_digest
    }

    pub fn scale_slope_digest_part_count(&self) -> usize {
        self.scale_slope_digest_part_count
    }

    pub fn performance_certification_digest(&self) -> &str {
        &self.performance_certification_digest
    }
}

struct SnapshotDigestParts<'a> {
    fixture_size: CausalInspectionScaleFixtureSize,
    artifact_identity: &'a CausalInspectionArtifactIdentity,
    evidence_reference_width: usize,
    anchor_derivation_slope_counter: usize,
    reference_resolution_slope_counter: usize,
    admission_slope_counter: usize,
    bridge_envelope_slope_counter: usize,
    materialization_slope_counter: usize,
    artifact_serialization_slope_counter: usize,
    bridge_unindexed_scan_count: usize,
    bridge_readmission_proof_identity: Option<&'a WorthQueryEvidenceIdentity>,
}

fn snapshot_identity(
    parts: SnapshotDigestParts<'_>,
) -> Result<CausalInspectionPerformanceSnapshotIdentity, IdentityDerivationDenial> {
    Ok(compose_causal_performance_snapshot_identity(
        parts.fixture_size,
        parts.artifact_identity,
        parts.evidence_reference_width,
        parts.anchor_derivation_slope_counter,
        parts.reference_resolution_slope_counter,
        parts.admission_slope_counter,
        parts.bridge_envelope_slope_counter,
        parts.materialization_slope_counter,
        parts.artifact_serialization_slope_counter,
        parts.bridge_unindexed_scan_count,
        parts.bridge_readmission_proof_identity,
    )?)
}

fn slope_identity(
    label: &str,
    small: usize,
    medium: usize,
    large: usize,
) -> Result<CausalInspectionPerformanceSlopeIdentity, IdentityDerivationDenial> {
    Ok(compose_causal_performance_slope_identity(
        label, small, medium, large,
    )?)
}
