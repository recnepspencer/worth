use std::sync::Arc;

use crate::commit_strategies::data::execution::executor_registration::CommitStrategyExecutor;
use crate::commit_strategies::data::execution::observation_context::StrategyObservationContext;
use crate::commit_strategies::data::CanonicalStrategyCommitRequest;

pub(crate) struct BoundStrategyExecution<'runtime> {
    pub(super) request: &'runtime CanonicalStrategyCommitRequest,
    pub(super) executor: Arc<dyn CommitStrategyExecutor>,
    pub(super) observation: StrategyObservationContext<'runtime>,
}
