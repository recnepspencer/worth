use crate::contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary;
use crate::contracts::runtime::operation_control::RelationalOperationControl;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionAllocationPolicy;

/// The original operation and retention binding selected by the transaction owner.
pub struct InvariantPreparationSource<'a> {
    pub operation: &'a RelationalOperationControl,
    pub retention: &'a worth_relational_branch::internal_port::history::retention::lane::RelationalBranchRetentionBinding,
}

/// Control for complete scope preparation. It does not charge the scope's
/// existing BTree/Vec heaps or mint execution authority.
pub struct InvariantPreparationControl<'scope, 'authority> {
    operation: RelationalOperationControl,
    allocation: ExecutionAllocationPolicy<'scope, 'authority>,
    boundary: RelationalInterruptionBoundary,
    retention: Option<worth_relational_branch::internal_port::history::retention::lane::RelationalBranchRetentionBinding>,
}

impl<'scope, 'authority> InvariantPreparationControl<'scope, 'authority> {
    #[cfg(test)]
    pub(crate) fn uninterrupted_proposal_validation() -> Self {
        Self::new(
            RelationalOperationControl::uninterrupted(),
            ExecutionAllocationPolicy::SystemAllocation,
            RelationalInterruptionBoundary::ProposalValidation,
        )
    }

    pub fn new(
        operation: RelationalOperationControl,
        allocation: ExecutionAllocationPolicy<'scope, 'authority>,
        boundary: RelationalInterruptionBoundary,
    ) -> Self {
        Self {
            operation,
            allocation,
            boundary,
            retention: None,
        }
    }

    pub fn for_transaction<'a>(
        source: impl Into<InvariantPreparationSource<'a>>,
        allocation: ExecutionAllocationPolicy<'scope, 'authority>,
        boundary: RelationalInterruptionBoundary,
    ) -> Self {
        let source = source.into();
        Self {
            operation: source.operation.clone(),
            allocation,
            boundary,
            retention: Some(source.retention.clone()),
        }
    }

    pub fn check(&self) -> Result<(), TransactionCommitError> {
        if let Some(interruption) = self.operation.observe(self.boundary) {
            if let Some(retention) = &self.retention {
                retention.record_interruption(interruption);
            }
            return Err(TransactionCommitError::interrupted(interruption));
        }
        self.allocation.check_live().map_err(|denial| {
            TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::ExecutionAllocationDenied { denial },
            ))
        })
    }
}
