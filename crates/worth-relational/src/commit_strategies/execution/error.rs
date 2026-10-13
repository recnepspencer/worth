use std::sync::Arc;

use crate::commit_strategies::data::{
    CommitStrategyDescriptorDigest, CommitStrategyId, StrategyExecutorFailure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyExecutionError {
    Execution(crate::execution::RelationalExecutionDenialCause),
    UnknownStrategyId {
        strategy_id: CommitStrategyId,
    },
    UnboundStrategyExecutor {
        strategy_id: CommitStrategyId,
    },
    DescriptorDigestMismatch {
        strategy_id: CommitStrategyId,
        request_digest: CommitStrategyDescriptorDigest,
        bound_digest: CommitStrategyDescriptorDigest,
    },
    UnsupportedReadContract {
        strategy_id: CommitStrategyId,
        detail: Arc<str>,
    },
    UnknownSnapshot {
        snapshot_id: crate::snapshots::data::SnapshotId,
    },
    ExecutorFailed {
        strategy_id: CommitStrategyId,
        failure: StrategyExecutorFailure,
    },
    ExecutorPanicked {
        strategy_id: CommitStrategyId,
    },
}
