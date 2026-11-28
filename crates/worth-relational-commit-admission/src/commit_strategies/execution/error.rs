use std::sync::Arc;

use crate::commit_strategies::data::{
    CommitStrategyDescriptorDigest, CommitStrategyId, StrategyExecutorFailure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyExecutionError {
    Execution(worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause),
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
        snapshot_id: worth_relational_contracts::internal_port::snapshots::data::coordinates::SnapshotId,
    },
    ExecutorFailed {
        strategy_id: CommitStrategyId,
        failure: StrategyExecutorFailure,
    },
    ExecutorPanicked {
        strategy_id: CommitStrategyId,
    },
}
