//! Actual merge and external-write preflight on an isolated fork of Query's current basis.
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{BranchId, RelationalCommitReceipt},
    identity::data::EntityId,
    runtime::RelationalRuntime,
    transactions::data::*,
};
use worth_execution::ExecutionRequest;
impl RelationalRuntime {
    /// Both bases and all publication facts come from real installed Query branches.
    /// The merged fork is private to this court; the application branch is untouched.
    #[doc(hidden)]
    pub fn probe_current_owned_member_after_merge_for_test(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &AdmittedRelationalBranchBasis,
        other: &AdmittedRelationalBranchBasis,
        owner: EntityId,
    ) -> Result<(RelationalCommitReceipt, Result<(), TransactionCommitError>), TransactionCommitError>
    {
        let setup = |detail| {
            crate::mvcc::origin_denial(format!("actual ownership merge court: {detail:?}"))
        };
        let shadow = BranchId(format!(
            "owned-merge-court-{}-{}",
            owner.slot_index(),
            current.descriptor().branch_id().0
        ));
        let (_, source) = self
            .observe_fork_source(current.descriptor().branch_id())
            .map_err(|cause| setup(format!("{cause:?}")))?;
        self.fork_branch(execution, shadow.clone(), source)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let request = crate::merge::data::MergeExecutionRequest::new(
            shadow.clone(),
            other.descriptor().branch_id().clone(),
            crate::merge::data::MergeIntent::ReconcileIntoTarget,
        );
        #[cfg(not(test))]
        let request = self
            .bind_merge_execution_request(request)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let prepared = self
            .prepare_merge_execution(execution, request)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let merged = self
            .execute_prepared_merge(execution, prepared)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let identity = self
            .branch_identity(&shadow)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let (_, basis) = self
            .observe_branch(&identity)
            .map_err(|cause| setup(format!("{cause:?}")))?;
        let roster = super::select_current(execution, &basis, &[RecordRef::Entity(owner)])?;
        let member = roster
            .origins()
            .find(|origin| origin.meaning.source_scope == owner)
            .and_then(|origin| origin.outputs.first())
            .copied()
            .ok_or_else(|| setup("current member is absent".to_owned()))?;
        let mut transaction = self
            .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| setup(format!("{cause:?}")))?;
        transaction
            .push_batch(
                WorkerIntentBatch::new("external-owned-delete-after-merge").push(
                    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                        entity_id: member,
                    })),
                ),
                execution.allocation_policy(),
            )
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
        Ok((
            merged.commit.commit.clone(),
            self.prepare_branch_transaction(execution, transaction)
                .map(|_| ()),
        ))
    }
}
