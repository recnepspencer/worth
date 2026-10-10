use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::publication::DeferredPublicationSettlementError;

use super::{WorthQueryRuntime, WorthQueryWorkspace};
use crate::ordinary::workflow::WorthQueryBranchMergeSettlementDeferred;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorthQuerySettlementRepairError {
    RelationalOwnerUnavailable,
    ForeignPhase(
        worth_query_execution::facade::application_contribution::WorthQueryForeignAdvancementPhase,
    ),
    PrimaryGraphIndexRefresh(
        worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenial,
    ),
    Settlement(DeferredPublicationSettlementError),
}

impl From<DeferredPublicationSettlementError> for WorthQuerySettlementRepairError {
    fn from(error: DeferredPublicationSettlementError) -> Self {
        Self::Settlement(error)
    }
}

impl std::fmt::Display for WorthQuerySettlementRepairError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignPhase(cause) => write!(formatter, "foreign advancement phase: {cause:?}"),
            Self::RelationalOwnerUnavailable => {
                formatter.write_str("this Query runtime has no Relational publication owner")
            }
            Self::PrimaryGraphIndexRefresh(error) => {
                write!(formatter, "primary graph index refresh failed: {error}")
            }
            Self::Settlement(error) => {
                write!(formatter, "publication settlement failed: {error:?}")
            }
        }
    }
}

impl std::error::Error for WorthQuerySettlementRepairError {}

impl WorthQueryRuntime {
    pub fn repair_deferred_branch_merge_settlement(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deferred: &WorthQueryBranchMergeSettlementDeferred,
    ) -> Result<RelationalCommitReceipt, WorthQuerySettlementRepairError> {
        self.backend
            .repair_deferred_branch_merge_settlement(execution, deferred)
    }

    pub fn repair_pending_branch_merge_settlement(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<RelationalCommitReceipt, WorthQuerySettlementRepairError> {
        self.backend
            .repair_pending_branch_merge_settlement(execution, commit_id)
    }
}

impl WorthQueryWorkspace {
    pub fn repair_deferred_branch_merge_settlement(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deferred: &WorthQueryBranchMergeSettlementDeferred,
    ) -> Result<RelationalCommitReceipt, WorthQuerySettlementRepairError> {
        self.runtime
            .repair_deferred_branch_merge_settlement(execution, deferred)
    }

    pub fn repair_pending_branch_merge_settlement(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: worth_relational::facade::history::CommitId,
    ) -> Result<RelationalCommitReceipt, WorthQuerySettlementRepairError> {
        self.runtime
            .repair_pending_branch_merge_settlement(execution, commit_id)
    }
}
