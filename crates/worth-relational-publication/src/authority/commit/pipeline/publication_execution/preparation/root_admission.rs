//! Settle root reconstruction once into the existing commit Work account.
use crate::mvcc::publication::authority::PreparedRelationalPublication;
use crate::mvcc::publication::PublicationPreparationDenial as Denial;
use contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use contracts::execution::request_work::RequestWorkBudget;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn run(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&RequestWorkBudget>,
    operation: impl FnOnce() -> Result<PreparedRelationalPublication, Denial>,
) -> Result<PreparedRelationalPublication, Denial> {
    let budget = match (
        contracts::execution::request_custody::request_lease(execution),
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
