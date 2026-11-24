use crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary;
use crate::runtime::operation_control::RelationalOperationControl;
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionAllocationPolicy;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

/// The original operation and retention binding selected by the transaction owner.
pub(crate) struct InvariantPreparationSource<'a> {
    pub(crate) operation: &'a RelationalOperationControl,
    pub(crate) retention: &'a crate::history::retention::RelationalBranchRetentionBinding,
}

/// Control for complete scope preparation. It does not charge the scope's
/// existing BTree/Vec heaps or mint execution authority.
pub(crate) struct InvariantPreparationControl<'scope, 'authority> {
    operation: RelationalOperationControl,
    allocation: ExecutionAllocationPolicy<'scope, 'authority>,
    boundary: RelationalInterruptionBoundary,
    retention: Option<crate::history::retention::RelationalBranchRetentionBinding>,
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

    pub(crate) fn new(
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

    pub(crate) fn for_transaction<'a>(
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

    pub(crate) fn check(&self) -> Result<(), TransactionCommitError> {
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
