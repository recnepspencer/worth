use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_foundational::{MaterializedFoundationalProfileSet, SupportProfiledArtifact};

use crate::facade::merge::{
    NormalizedRelationalMergeRequest, RelationalMergeCorrespondenceWitness,
    RelationalMergeProofPacket, RelationalMergeStrategyWitness,
    RelationalSchemaReconciliationWitness,
};
use crate::history::data::RelationalMergeBranchBasis;

use super::merge_support_rows::RelationalMergeSupportInspectionRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalMergeSupportInspectionDenial {
    IllegalSupportProfile(String),
    RequestedProfileAdmissionDenied,
    RequestedProfileAdmissionDeferred,
    RequestedProfileAdmissionStale,
    RequestedProfileAdmissionRebindRequired,
    RequestedProfileAdmissionFailed,
    SupportAttachmentDenied,
    SupportAttachmentDeferred,
    SupportAttachmentStale,
    SupportAttachmentRebindRequired,
    SupportAttachmentFailed,
    InconsistentRetainedProofAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RelationalMergeSupportInspectionInput {
    pub request: NormalizedRelationalMergeRequest,
    pub branch_basis: RelationalMergeBranchBasis,
    pub proof_packet: Option<RelationalMergeProofPacket>,
    pub correspondence_witness: Option<RelationalMergeCorrespondenceWitness>,
    pub schema_reconciliation_witness: Option<RelationalSchemaReconciliationWitness>,
    pub strategy_witness: Option<RelationalMergeStrategyWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeSupportInspectionSurface {
    request_digest: String,
    branch_basis_digest: String,
    rows: Arc<[RelationalMergeSupportInspectionRow]>,
}

impl RelationalMergeSupportInspectionSurface {
    pub(crate) fn retained(
        request_digest: String,
        branch_basis_digest: String,
        rows: Arc<[RelationalMergeSupportInspectionRow]>,
    ) -> Self {
        Self {
            request_digest,
            branch_basis_digest,
            rows,
        }
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn branch_basis_digest(&self) -> &str {
        &self.branch_basis_digest
    }

    pub fn rows(&self) -> &[RelationalMergeSupportInspectionRow] {
        &self.rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMergeSupportInspectionWitness {
    support_artifact: SupportProfiledArtifact<RelationalMergeSupportInspectionSurface>,
    witness_digest: String,
}

impl RelationalMergeSupportInspectionWitness {
    pub(crate) fn retained(
        support_artifact: SupportProfiledArtifact<RelationalMergeSupportInspectionSurface>,
    ) -> Self {
        let witness_digest = support_witness_digest(
            support_artifact.payload().payload().request_digest(),
            support_artifact.payload().payload().branch_basis_digest(),
            support_artifact.payload().payload().rows(),
            support_artifact.payload().profile(),
        );
        Self {
            support_artifact,
            witness_digest,
        }
    }

    pub fn support_artifact(
        &self,
    ) -> &SupportProfiledArtifact<RelationalMergeSupportInspectionSurface> {
        &self.support_artifact
    }

    pub fn profile(&self) -> &MaterializedFoundationalProfileSet {
        self.support_artifact.payload().profile()
    }

    pub fn rows(&self) -> &[RelationalMergeSupportInspectionRow] {
        self.support_artifact.payload().payload().rows()
    }

    pub fn witness_digest(&self) -> &str {
        &self.witness_digest
    }
}

fn support_witness_digest(
    request_digest: &str,
    branch_basis_digest: &str,
    rows: &[RelationalMergeSupportInspectionRow],
    profile: &MaterializedFoundationalProfileSet,
) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.relational.merge_support.witness.v1");
    bytes.extend_from_slice(request_digest.as_bytes());
    bytes.extend_from_slice(branch_basis_digest.as_bytes());
    bytes.extend_from_slice(
        &rmp_serde::to_vec_named(rows).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "support witness rows must encode"),
    );
    append_profile_bytes(&mut bytes, profile);
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn append_profile_bytes(bytes: &mut Vec<u8>, profile: &MaterializedFoundationalProfileSet) {
    append_profile_set_bytes(bytes, profile.requested());
    append_profile_set_bytes(bytes, profile.admitted());
    append_profile_set_bytes(bytes, profile.materialized());
    append_narrowing_bytes(bytes, profile.requested_to_admitted_narrowing());
    append_narrowing_bytes(bytes, profile.admitted_to_materialized_narrowing());
}

fn append_profile_set_bytes(
    bytes: &mut Vec<u8>,
    profile: &worth_foundational::FoundationalProfileSet,
) {
    bytes.push(match profile.diagnostic_richness() {
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal => 1,
        worth_foundational::DiagnosticRichnessProfile::Standard => 2,
        worth_foundational::DiagnosticRichnessProfile::Forensic => 3,
    });
    bytes.push(match profile.support_posture() {
        worth_foundational::SupportPostureProfile::InternalOnly => 1,
        worth_foundational::SupportPostureProfile::SupportReady => 2,
        worth_foundational::SupportPostureProfile::CertificationReady => 3,
    });
    bytes.push(match profile.compatibility_posture() {
        worth_foundational::CompatibilityPostureProfile::NativeOnly => 1,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered => 2,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired => 3,
    });
    bytes.push(match profile.admission_readiness() {
        worth_foundational::AdmissionReadinessProfile::CandidateOnly => 1,
        worth_foundational::AdmissionReadinessProfile::Admitted => 2,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady => 3,
    });
    bytes.push(match profile.retention_delivery() {
        worth_foundational::RetentionDeliveryProfile::Ephemeral => 1,
        worth_foundational::RetentionDeliveryProfile::Retained => 2,
        worth_foundational::RetentionDeliveryProfile::Durable => 3,
    });
    bytes.push(match profile.certification_posture() {
        worth_foundational::CertificationPostureProfile::Uncertified => 1,
        worth_foundational::CertificationPostureProfile::EvidenceBacked => 2,
        worth_foundational::CertificationPostureProfile::ProductionCertified => 3,
    });
}

fn append_narrowing_bytes(
    bytes: &mut Vec<u8>,
    narrowing: Option<worth_foundational::FoundationalProfileNarrowingRecord>,
) {
    match narrowing {
        None => bytes.push(0),
        Some(record) => {
            bytes.push(1);
            bytes.push(match record.kind() {
                worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced => 1,
                worth_foundational::FoundationalProfileNarrowingKind::RetentionNarrowed => 2,
                worth_foundational::FoundationalProfileNarrowingKind::SupportPostureReduced => 3,
                worth_foundational::FoundationalProfileNarrowingKind::CertificationPostureReduced => 4,
                worth_foundational::FoundationalProfileNarrowingKind::CompatibilityRestricted => 5,
            });
            bytes.extend_from_slice(record.reason().as_bytes());
        }
    }
}
