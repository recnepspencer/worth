use std::fmt;
use std::sync::Arc;

use super::execution_draft::StrategyExecutionResult;
use super::observation_context::StrategyObservationContext;
use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CommitStrategyDescriptor, CommitStrategyDescriptorDigest,
    CommitStrategyId, StrategyExecutorFailure,
};

pub trait CommitStrategyExecutor: Send + Sync + 'static {
    fn execute(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &CanonicalStrategyCommitRequest,
        observation: &StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure>;
}

#[derive(Clone)]
pub struct CommitStrategyExecutionRegistration {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    executor: Arc<dyn CommitStrategyExecutor>,
}

impl fmt::Debug for CommitStrategyExecutionRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommitStrategyExecutionRegistration")
            .field("strategy_id", &self.strategy_id)
            .field("descriptor_digest", &self.descriptor_digest)
            .finish()
    }
}

impl CommitStrategyExecutionRegistration {
    pub fn new<E>(descriptor: &CommitStrategyDescriptor, executor: E) -> Self
    where
        E: CommitStrategyExecutor,
    {
        Self {
            strategy_id: descriptor.id(),
            descriptor_digest: descriptor.digest(),
            executor: Arc::new(executor),
        }
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub(crate) fn executor(&self) -> Arc<dyn CommitStrategyExecutor> {
        Arc::clone(&self.executor)
    }
}
