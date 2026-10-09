use crate::mvcc::{RelationalInterruptionBoundary, RelationalOperationControl};
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};
use worth_execution::ExecutionAllocationPolicy;

/// Control for complete scope preparation. It does not charge the scope's
/// existing BTree/Vec heaps or mint execution authority.
pub(crate) struct InvariantPreparationControl<'scope, 'authority> {
    operation: RelationalOperationControl,
    allocation: ExecutionAllocationPolicy<'scope, 'authority>,
    boundary: RelationalInterruptionBoundary,
    retention: Option<crate::history::retention::RelationalBranchRetentionBinding>,
}

impl<'scope, 'authority> InvariantPreparationControl<'scope, 'authority> {
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

    pub(crate) fn for_transaction(
        input: &crate::mvcc::RelationalTransactionValidationInput,
        allocation: ExecutionAllocationPolicy<'scope, 'authority>,
        boundary: RelationalInterruptionBoundary,
    ) -> Self {
        Self {
            operation: input.control().clone(),
            allocation,
            boundary,
            retention: Some(input.basis().inner.retention_binding.clone()),
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
