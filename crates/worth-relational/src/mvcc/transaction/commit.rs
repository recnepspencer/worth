use crate::runtime::RelationalRuntime;
use crate::transactions::data::{CommitResult, TransactionCommitError};

impl RelationalRuntime {
    pub fn commit_branch_transaction(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<CommitResult, TransactionCommitError> {
        let candidate = self.prepare_branch_transaction(transaction, allocation_policy)?;
        self.publish_prepared_candidate(candidate)
    }

    pub fn commit_branch_transaction_with_lease(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<CommitResult, TransactionCommitError> {
        let candidate = self.prepare_branch_transaction_with_lease(transaction, lease)?;
        self.publish_prepared_candidate(candidate)
    }

    pub fn prepare_branch_transaction(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        self.preparation_port()
            .prepare_branch_transaction(transaction, allocation_policy)
    }

    pub fn prepare_branch_transaction_with_lease(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
        self.preparation_port()
            .prepare_branch_transaction_with_lease(transaction, lease)
    }

    /// The independently borrowable preparation service for this runtime.
    ///
    /// Preparation is one of four owner services obtained from a shared borrow
    /// of the runtime: preparation here,
    /// [`RelationalRuntime::fork_port`](crate::runtime::RelationalRuntime::fork_port)
    /// for branch creation,
    /// [`RelationalRuntime::publication_port`](crate::runtime::RelationalRuntime::publication_port)
    /// for linearization, and
    /// [`RelationalRuntime::settlement_port`](crate::runtime::RelationalRuntime::settlement_port)
    /// for durability and repair. Each is `Clone + Send + Sync`, so unrelated
    /// branches can progress at once without any caller excluding another.
    ///
    /// Preparation performs the fallible work *before* effects, and a prepared
    /// candidate that is never published moves no public reference:
    ///
    /// ```
    /// use worth_relational::facade::mvcc::RelationalTransactionIntent;
    /// use worth_relational::facade::runtime::RelationalRuntimeApi;
    /// use worth_relational::facade::schema::RelationalSchemaRegistry;
    /// use worth_relational::facade::transactions::WorkerIntentBatch;
    ///
    /// let runtime = RelationalRuntimeApi::builder()
    ///     .schema_registry(RelationalSchemaRegistry::new())
    ///     .build();
    /// let preparation = runtime.preparation_port();
    ///
    /// let identity = runtime.main_branch_identity();
    /// let (before, basis) = runtime
    ///     .observe_branch(&identity)
    ///     .expect("an owner-issued identity observes its own branch");
    ///
    /// let mut transaction = runtime
    ///     .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
    ///     .expect("an exact admitted basis admits a branch transaction");
    /// transaction
    ///     .push_batch(WorkerIntentBatch::new("preparation-port-doc"), worth_execution::ExecutionAllocationPolicy::SystemAllocation)
    ///     .expect("one empty batch stages within the declared budget");
    ///
    /// let candidate = preparation
    ///     .prepare_branch_transaction(transaction, worth_execution::ExecutionAllocationPolicy::SystemAllocation)
    ///     .expect("preparation validates without moving the branch");
    /// preparation
    ///     .discard_prepared_candidate(candidate)
    ///     .expect("a candidate is consumed exactly once, by discard or publication");
    ///
    /// // Discarding released the candidate's retained residue and moved nothing:
    /// // the branch still observes the exact same target and generation.
    /// let (after, _basis) = runtime
    ///     .observe_branch(&identity)
    ///     .expect("the branch is still observable");
    /// assert_eq!(after, before);
    /// ```
    ///
    /// The runnable owner workflow, including concurrent publication on
    /// unrelated branches, is `examples/branch_local_mvcc.rs`.
    pub fn preparation_port(&self) -> crate::mvcc::RelationalPreparationPort {
        crate::mvcc::RelationalPreparationPort::new(
            crate::runtime::RelationalPreparationOwnerBinding::from_runtime(self),
        )
    }
}
