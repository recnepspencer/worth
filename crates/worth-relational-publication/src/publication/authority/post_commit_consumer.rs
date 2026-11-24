//! Replaceable post-commit publication consumer boundary.

use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::snapshots::data::coordinates::SnapshotId;

#[derive(Debug, Clone, Copy)]
pub struct PostCommitConsumptionContext {
    commit_id: CommitId,
    snapshot_id: SnapshotId,
}

impl PostCommitConsumptionContext {
    pub(crate) const fn new(commit_id: CommitId, snapshot_id: SnapshotId) -> Self {
        Self {
            commit_id,
            snapshot_id,
        }
    }

    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub const fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCommitConsumptionFailure {
    ConsumerFailureNonAuthoritative,
}

pub trait PostCommitConsumer: std::fmt::Debug + Send + Sync {
    fn consume(
        &self,
        context: &PostCommitConsumptionContext,
    ) -> Result<(), PostCommitConsumptionFailure>;
}

#[derive(Debug)]
struct NoPostCommitConsumerFailure;

impl PostCommitConsumer for NoPostCommitConsumerFailure {
    fn consume(
        &self,
        _context: &PostCommitConsumptionContext,
    ) -> Result<(), PostCommitConsumptionFailure> {
        Ok(())
    }
}

pub(crate) fn production_post_commit_consumer() -> std::sync::Arc<dyn PostCommitConsumer> {
    std::sync::Arc::new(NoPostCommitConsumerFailure)
}
