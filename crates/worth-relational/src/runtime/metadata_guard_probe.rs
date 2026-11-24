//! Bind the closed metadata-write court to this runtime's ordinary preparation door.
use super::RelationalRuntime;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::{EntityId, KindId};
use crate::transactions::data::outcomes::transaction_commit_error::{
    origin_denial, TransactionCommitError,
};
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    #[doc(hidden)]
    pub fn probe_current_metadata_writes_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<Vec<(KindId, u8, Result<(), TransactionCommitError>)>, TransactionCommitError> {
        crate::publication::ownership::metadata_guard_court::inspect_metadata_writes(
            execution,
            basis,
            owner,
            |batch| {
                let mut transaction = self
                    .begin_branch_transaction(
                        basis,
                        crate::mvcc::RelationalTransactionIntent::ordinary(),
                    )
                    .map_err(|cause| origin_denial(format!("court transaction: {cause:?}")))?;
                transaction
                    .push_batch(batch, execution.allocation_policy())
                    .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
                Ok(self
                    .prepare_branch_transaction(execution, transaction)
                    .map(|_| ()))
            },
        )
    }
}
