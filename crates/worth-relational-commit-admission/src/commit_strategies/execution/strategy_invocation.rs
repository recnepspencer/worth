use worth_relational_branch::internal_port::commit_strategies::data::execution::observation_context::StrategyObservationContextOwnerPort as _;
use worth_relational_contracts::internal_port::commit_strategies::data::execution::execution_draft::StrategyExecutionDraftOwnerPort as _;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::commit_strategies::data::StrategyExecutionDraft;

use super::bound_strategy::BoundStrategyExecution;
use super::error::StrategyExecutionError;

pub fn execute_bound_strategy(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bound: BoundStrategyExecution<'_>,
) -> Result<StrategyExecutionDraft, StrategyExecutionError> {
    let strategy_id = bound.request.strategy_id();
    let result = catch_unwind(AssertUnwindSafe(|| {
        bound
            .executor
            .execute(execution, bound.request, &bound.observation)
    }));
    match result {
        Ok(Ok(strategy_result)) => Ok(StrategyExecutionDraft::from_measured_result(
            bound.request,
            strategy_result,
            bound.observation.measured_summary(),
        )),
        Ok(Err(failure)) => Err(StrategyExecutionError::ExecutorFailed {
            strategy_id,
            failure,
        }),
        Err(_) => Err(StrategyExecutionError::ExecutorPanicked { strategy_id }),
    }
}
