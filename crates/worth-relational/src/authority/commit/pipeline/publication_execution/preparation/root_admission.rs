//! Settle root reconstruction once into the existing commit Work account.
use crate::execution::{RelationalExecutionDenialCause as Cause, RequestWorkBudget};
use crate::mvcc::{PreparedRelationalPublication, PublicationPreparationDenial as Denial};

pub(super) fn run(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&RequestWorkBudget>,
    operation: impl FnOnce() -> Result<PreparedRelationalPublication, Denial>,
) -> Result<PreparedRelationalPublication, Denial> {
    let budget = match (
        crate::authority::commit::pipeline::request_lease(execution),
        budget,
    ) {
        (Some(_), Some(budget)) => budget,
        _ => return operation(),
    };
    let mut result = None;
    let handoff = budget.run_stage(*execution, || result = Some(operation()));
    match (handoff, result) {
        (Ok(()), Some(result)) => result,
        (Ok(()), None) => Err(Denial::Invalid(
            "root admission callback did not execute".into(),
        )),
        (Err(cause), Some(Err(inner))) if Cause::from(cause) == Cause::NestedStopped => Err(inner),
        (Err(cause), _) => Err(Denial::Execution(cause.into())),
    }
}
