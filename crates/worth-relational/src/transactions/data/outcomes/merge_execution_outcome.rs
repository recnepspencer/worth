use super::plan_artifacts::{MergeExecutionStructuralSummary, MergeExecutionSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeExecutionOutcome {
    pub commit: super::commit_result::CommitResult,
    pub execution_summary: MergeExecutionSummary,
    pub structural_summary: MergeExecutionStructuralSummary,
}
