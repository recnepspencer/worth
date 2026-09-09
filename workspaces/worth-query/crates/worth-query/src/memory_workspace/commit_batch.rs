use super::{WorthQueryMemoryWorkspace, WorthQuerySnapshotIdentity, WorthQueryWorkspaceError};
use worth_relational::facade::transactions::{CommitResult, WorkerIntentBatch};

impl WorthQueryMemoryWorkspace {
    pub(super) fn commit_batch(
        &self,
        batch: WorkerIntentBatch,
    ) -> Result<(CommitResult, WorthQuerySnapshotIdentity), WorthQueryWorkspaceError> {
        self.runtime.with_runtime_mut(|runtime| {
            let branch = runtime.main_branch_identity();
            let basis = runtime
                .admit_branch_basis(&branch)
                .map_err(super::transaction_denial::basis)?;
            let mut transaction = runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .map_err(super::transaction_denial::admission)?;
            transaction
                .push_batch(batch)
                .map_err(super::transaction_denial::staging)?;
            let result = transaction
                .commit(runtime)
                .map_err(super::transaction_denial::commit)?;
            let snapshot = super::runtime_identity::snapshot_identity_from_runtime(runtime);
            super::commit_snapshot_closeout::release_commit_snapshot(runtime, &result.snapshot);
            Ok((result, snapshot))
        })
    }

    pub(crate) fn relational_source_owner(
        &self,
    ) -> worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner {
        self.runtime.clone()
    }
}
