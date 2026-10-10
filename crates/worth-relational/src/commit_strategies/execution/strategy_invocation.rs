use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::commit_strategies::data::StrategyExecutionDraft;

use super::bound_execution::BoundStrategyExecution;
use super::error::StrategyExecutionError;

pub(crate) fn execute_bound_strategy(
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
