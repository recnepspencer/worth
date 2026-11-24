//! Bind the ownership probe to the receiver's installed fork and merge services.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::BranchId;
use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::mvcc::transaction::staging::RelationalTransactionStagingDenialOwnerPort as _;
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
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(format!(
                "actual ownership merge court: {detail:?}"
            ))
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
        let member = crate::publication::ownership::merge_custody::selected_member(
            execution, &basis, owner,
        )?
        .ok_or_else(|| setup("current member is absent".to_owned()))?;
        let mut transaction = self
            .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
            .map_err(|cause| setup(format!("{cause:?}")))?;
        transaction
            .push_batch(
                crate::publication::ownership::merge_custody::delete_member_batch(member),
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
