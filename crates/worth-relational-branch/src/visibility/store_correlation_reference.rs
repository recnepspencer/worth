use crate::history::data::{BranchId, CommitId};
use crate::snapshots::data::{SnapshotHandle, SnapshotId};
use crate::transactions::data::primitives::TransactionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationalStoreCorrelationReferenceKind {
    Transaction,
    Branch,
    Snapshot,
    Projection,
    CurrentBasis,
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalStoreCorrelationReference {
    kind: RelationalStoreCorrelationReferenceKind,
    runtime_instance_id: u64,
    semantic_id: String,
}

impl RelationalStoreCorrelationReference {
    pub fn transaction(runtime_instance_id: u64, transaction_id: TransactionId) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::Transaction,
            runtime_instance_id,
            transaction_id.0.to_string(),
        )
    }

    pub fn branch(runtime_instance_id: u64, branch_id: BranchId) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::Branch,
            runtime_instance_id,
            branch_id.0,
        )
    }

    pub fn snapshot_handle(handle: &SnapshotHandle) -> Self {
        Self::snapshot(handle.runtime_instance_id, handle.snapshot_id)
    }

    pub fn snapshot(runtime_instance_id: u64, snapshot_id: SnapshotId) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::Snapshot,
            runtime_instance_id,
            snapshot_id.0.to_string(),
        )
    }

    pub fn projection(runtime_instance_id: u64, projection_id: impl Into<String>) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::Projection,
            runtime_instance_id,
            projection_id,
        )
    }

    pub fn current_basis(runtime_instance_id: u64, basis_id: impl Into<String>) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::CurrentBasis,
            runtime_instance_id,
            basis_id,
        )
    }

    pub fn commit(runtime_instance_id: u64, commit_id: CommitId) -> Self {
        Self::new(
            RelationalStoreCorrelationReferenceKind::Commit,
            runtime_instance_id,
            commit_id.0.to_string(),
        )
    }

    fn new(
        kind: RelationalStoreCorrelationReferenceKind,
        runtime_instance_id: u64,
        semantic_id: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            runtime_instance_id,
            semantic_id: semantic_id.into(),
        }
    }

    pub const fn kind(&self) -> RelationalStoreCorrelationReferenceKind {
        self.kind
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn semantic_id(&self) -> &str {
        &self.semantic_id
    }
}
