//! Borrowed owner access across the sealed Relational package boundary.
use super::*;

#[doc(hidden)]
pub trait RelationalMergeSupportInspectionRowOwnerPort: Sized {
    fn branch_basis(
        basis_digest: String,
        source_head_commit_id: crate::contracts::history::data::coordinates::CommitId,
        target_head_commit_id: crate::contracts::history::data::coordinates::CommitId,
        merge_base_commit_id: crate::contracts::history::data::coordinates::CommitId,
    ) -> Self;

    fn request_admission(
        request_digest: String,
        packet_digest: Option<String>,
        admission_posture: Option<RelationalMergeProofPacketAdmissionPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self;

    fn correspondence(
        witness_digest: Option<String>,
        admitted_count: usize,
        denied_count: usize,
        unavailable_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalMergeCorrespondenceWitnessPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self;

    fn schema(
        witness_digest: Option<String>,
        reconciled_count: usize,
        denied_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalSchemaReconciliationWitnessPosture>,
        sample_decision_boundary: Option<MergePolicyDecisionBoundary>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self;

    fn strategy(
        witness_digest: Option<String>,
        aspect_policy_count: usize,
        topology_count: usize,
        deletion_count: usize,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self;

    fn compatibility(
        posture: RelationalMergeSupportInspectionCompatibilityPosture,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self;
}

impl RelationalMergeSupportInspectionRowOwnerPort for RelationalMergeSupportInspectionRow {
    fn branch_basis(
        basis_digest: String,
        source_head_commit_id: crate::contracts::history::data::coordinates::CommitId,
        target_head_commit_id: crate::contracts::history::data::coordinates::CommitId,
        merge_base_commit_id: crate::contracts::history::data::coordinates::CommitId,
    ) -> Self {
        Self::branch_basis(
            basis_digest,
            source_head_commit_id,
            target_head_commit_id,
            merge_base_commit_id,
        )
    }

    fn request_admission(
        request_digest: String,
        packet_digest: Option<String>,
        admission_posture: Option<RelationalMergeProofPacketAdmissionPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        Self::request_admission(request_digest, packet_digest, admission_posture, absence)
    }

    fn correspondence(
        witness_digest: Option<String>,
        admitted_count: usize,
        denied_count: usize,
        unavailable_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalMergeCorrespondenceWitnessPosture>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        Self::correspondence(
            witness_digest,
            admitted_count,
            denied_count,
            unavailable_count,
            sample_record,
            sample_target_record,
            sample_posture,
            absence,
        )
    }

    fn schema(
        witness_digest: Option<String>,
        reconciled_count: usize,
        denied_count: usize,
        sample_record: Option<RecordRef>,
        sample_target_record: Option<RecordRef>,
        sample_posture: Option<RelationalSchemaReconciliationWitnessPosture>,
        sample_decision_boundary: Option<MergePolicyDecisionBoundary>,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        Self::schema(
            witness_digest,
            reconciled_count,
            denied_count,
            sample_record,
            sample_target_record,
            sample_posture,
            sample_decision_boundary,
            absence,
        )
    }

    fn strategy(
        witness_digest: Option<String>,
        aspect_policy_count: usize,
        topology_count: usize,
        deletion_count: usize,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        Self::strategy(
            witness_digest,
            aspect_policy_count,
            topology_count,
            deletion_count,
            absence,
        )
    }

    fn compatibility(
        posture: RelationalMergeSupportInspectionCompatibilityPosture,
        absence: Option<RelationalMergeSupportInspectionAbsenceKind>,
    ) -> Self {
        Self::compatibility(posture, absence)
    }
}
