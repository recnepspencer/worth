use std::sync::Arc;

use crate::commit_strategies::data::execution::executor_registration::CommitStrategyExecutor;
use worth_relational_branch::internal_port::commit_strategies::data::execution::observation_context::StrategyObservationContext;
use crate::commit_strategies::data::CanonicalStrategyCommitRequest;

pub struct BoundStrategyExecution<'runtime> {
    pub request: &'runtime CanonicalStrategyCommitRequest,
    pub executor: Arc<dyn CommitStrategyExecutor>,
    pub observation: StrategyObservationContext<'runtime>,
}
