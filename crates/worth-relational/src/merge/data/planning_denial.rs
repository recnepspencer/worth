use super::requests::denial::RelationalMergeRequestNormalizationDenial;
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::transactions::data::primitives::RecordRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergePlanningError {
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    RequestNormalization(RelationalMergeRequestNormalizationDenial),
    UnboundBranchCurrentness,
    SourceObservationDenied(crate::branch::basis_denial::RelationalBranchBasisDenial),
    TargetObservationDenied(crate::branch::basis_denial::RelationalBranchBasisDenial),
    MissingSourceHead {
        branch_id: BranchId,
    },
    MissingTargetHead {
        branch_id: BranchId,
    },
    MissingMergeBase {
        source_branch: BranchId,
        target_branch: BranchId,
    },
    /// The selected common ancestor has no indexed canonical commit receipt.
    MissingMergeBaseCommit {
        commit_id: CommitId,
    },
    /// A selected branch delta names a commit with no canonical history entry.
    MissingDeltaCommit {
        commit_id: CommitId,
    },
    MissingConflictSourceRecord {
        record: RecordRef,
    },
    MissingPolicySourceRecord {
        record: RecordRef,
    },
    MissingLoweringSourceRecord {
        record: RecordRef,
    },
    MissingLoweringConflictClassification {
        record: RecordRef,
    },
    MissingCausalAnnotation {
        record: RecordRef,
    },
    MissingLoweredRecordExecutionBundle {
        classification: crate::merge::data::conflicts::MergeConflictClass,
        readiness: crate::merge::data::policy::MergeExecutionReadiness,
        lowered_action: Option<crate::merge::data::policy::LoweredMergeAction>,
    },
    MissingLoweredRecordDenialBundle,
}

impl From<RelationalMergeRequestNormalizationDenial> for MergePlanningError {
    fn from(value: RelationalMergeRequestNormalizationDenial) -> Self {
        Self::RequestNormalization(value)
    }
}

impl From<crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial>
    for MergePlanningError
{
    fn from(
        value: crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial,
    ) -> Self {
        match value {
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::SourceObservationDenied(
                denial,
            ) => Self::SourceObservationDenied(denial),
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::TargetObservationDenied(
                denial,
            ) => Self::TargetObservationDenied(denial),
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::MissingSourceHead {
                branch_id,
            } => Self::MissingSourceHead { branch_id },
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::MissingTargetHead {
                branch_id,
            } => Self::MissingTargetHead { branch_id },
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::MissingMergeBase {
                source_branch,
                target_branch,
            } => Self::MissingMergeBase {
                source_branch,
                target_branch,
            },
            crate::history::data::merge_branch_basis::RelationalMergeBranchBasisDenial::MissingMergeBaseCommit {
                commit_id,
            } => Self::MissingMergeBaseCommit { commit_id },
        }
    }
}
