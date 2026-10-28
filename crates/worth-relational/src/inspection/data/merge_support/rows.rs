use crate::facade::merge::{
    MergePolicyDecisionBoundary, RelationalMergeCorrespondenceWitnessPosture,
    RelationalMergeProofPacketAdmissionPosture, RelationalSchemaReconciliationWitnessPosture,
};
use crate::transactions::data::RecordRef;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeSupportInspectionRowKind {
    BranchBasis,
    RequestAdmission,
    Correspondence,
    Schema,
    Strategy,
    Compatibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeSupportInspectionAbsenceKind {
    MissingProofPacket,
    MissingCorrespondenceWitness,
    MissingSchemaReconciliationWitness,
    MissingStrategyWitness,
    MissingCompatibilityWitnessPhaseDependency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeSupportInspectionCompatibilityPosture {
    UnavailablePhaseDependency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeSupportInspectionRow {
    BranchBasis {
        basis_digest: String,
        source_head_commit_id: crate::facade::history::CommitId,
        target_head_commit_id: crate::facade::history::CommitId,
        merge_base_commit_id: crate::facade::history::CommitId,
        row_digest: String,
    },
    RequestAdmission {
        request_digest: String,
        packet_digest: Option<String>,
        admission_posture: Option<RelationalMergeProofPacketAdmissionPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
        row_digest: String,
    },
    Correspondence {
        witness_digest: Option<String>,
        admitted_count: usize,
        denied_count: usize,
        unavailable_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalMergeCorrespondenceWitnessPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
        row_digest: String,
    },
    Schema {
        witness_digest: Option<String>,
        reconciled_count: usize,
        denied_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalSchemaReconciliationWitnessPosture>,
        sample_decision_boundary: Option<MergePolicyDecisionBoundary>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
        row_digest: String,
    },
    Strategy {
        witness_digest: Option<String>,
        aspect_policy_count: usize,
        topology_count: usize,
        deletion_count: usize,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
        row_digest: String,
    },
    Compatibility {
        posture: RelationalMergeSupportInspectionCompatibilityPosture,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
        row_digest: String,
    },
}

impl RelationalMergeSupportInspectionRow {
    pub fn kind(&self) -> RelationalMergeSupportInspectionRowKind {
        match self {
            Self::BranchBasis { .. } => RelationalMergeSupportInspectionRowKind::BranchBasis,
            Self::RequestAdmission { .. } => {
                RelationalMergeSupportInspectionRowKind::RequestAdmission
            }
            Self::Correspondence { .. } => RelationalMergeSupportInspectionRowKind::Correspondence,
            Self::Schema { .. } => RelationalMergeSupportInspectionRowKind::Schema,
            Self::Strategy { .. } => RelationalMergeSupportInspectionRowKind::Strategy,
            Self::Compatibility { .. } => RelationalMergeSupportInspectionRowKind::Compatibility,
        }
    }

    pub fn row_digest(&self) -> &str {
        match self {
            Self::BranchBasis { row_digest, .. }
            | Self::RequestAdmission { row_digest, .. }
            | Self::Correspondence { row_digest, .. }
            | Self::Schema { row_digest, .. }
            | Self::Strategy { row_digest, .. }
            | Self::Compatibility { row_digest, .. } => row_digest,
        }
    }

    pub(crate) fn branch_basis(
        basis_digest: String,
        source_head_commit_id: crate::facade::history::CommitId,
        target_head_commit_id: crate::facade::history::CommitId,
        merge_base_commit_id: crate::facade::history::CommitId,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.branch_basis.v1",
            &basis_digest,
            source_head_commit_id,
            target_head_commit_id,
            merge_base_commit_id,
        ));
        Self::BranchBasis {
            basis_digest,
            source_head_commit_id,
            target_head_commit_id,
            merge_base_commit_id,
            row_digest,
        }
    }

    pub(crate) fn request_admission(
        request_digest: String,
        packet_digest: Option<String>,
        admission_posture: Option<RelationalMergeProofPacketAdmissionPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.request_admission.v1",
            &request_digest,
            &packet_digest,
            admission_posture,
            absence,
        ));
        Self::RequestAdmission {
            request_digest,
            packet_digest,
            admission_posture,
            absence,
            row_digest,
        }
    }

    pub(crate) fn correspondence(
        witness_digest: Option<String>,
        admitted_count: usize,
        denied_count: usize,
        unavailable_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalMergeCorrespondenceWitnessPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.correspondence.v1",
            &witness_digest,
            admitted_count,
            denied_count,
            unavailable_count,
            &sample_record,
            &sample_target_record,
            sample_posture,
            absence,
        ));
        Self::Correspondence {
            witness_digest,
            admitted_count,
            denied_count,
            unavailable_count,
            sample_record,
            sample_target_record,
            sample_posture,
            absence,
            row_digest,
        }
    }

    pub(crate) fn schema(
        witness_digest: Option<String>,
        reconciled_count: usize,
        denied_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalSchemaReconciliationWitnessPosture>,
        sample_decision_boundary: Option<MergePolicyDecisionBoundary>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.schema.v1",
            &witness_digest,
            reconciled_count,
            denied_count,
            &sample_record,
            &sample_target_record,
            sample_posture,
            sample_decision_boundary,
            absence,
        ));
        Self::Schema {
            witness_digest,
            reconciled_count,
            denied_count,
            sample_record,
            sample_target_record,
            sample_posture,
            sample_decision_boundary,
            absence,
            row_digest,
        }
    }

    pub(crate) fn strategy(
        witness_digest: Option<String>,
        aspect_policy_count: usize,
        topology_count: usize,
        deletion_count: usize,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.strategy.v1",
            &witness_digest,
            aspect_policy_count,
            topology_count,
            deletion_count,
            absence,
        ));
        Self::Strategy {
            witness_digest,
            aspect_policy_count,
            topology_count,
            deletion_count,
            absence,
            row_digest,
        }
    }

    pub(crate) fn compatibility(
        posture: RelationalMergeSupportInspectionCompatibilityPosture,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        let row_digest = row_digest(&(
            "worth.relational.merge_support.compatibility.v1",
            posture,
            absence,
        ));
        Self::Compatibility {
            posture,
            absence,
            row_digest,
        }
    }
}

fn row_digest(value: &impl Serialize) -> String {
    let digest = Sha256::digest(rmp_serde::to_vec_named(value).expect(/* Proof: The serialized value contains only closed native scalar, sequence, map and derived-Serialize record fields; encoding uses an in-memory Vec/hash writer, with no user serializer or fallible I/O. */ "support row must encode"));
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
