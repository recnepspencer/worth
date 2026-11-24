//! Settle native content work into the original request account.
use crate::content::codec_control::final_denial;
use crate::content::denial::RelationalContentDenial as Denial;
use worth_execution::ExecutionRequest;

/// Top-level content evidence settles once into the existing commit account.
/// Kernels remain unaware of the dispatcher and cannot recursively borrow it.
pub fn run_commit_stage<T>(
    execution: &ExecutionRequest<'_, '_>,
    budget: Option<&super::request_work::RequestWorkBudget>,
    operation: impl FnOnce() -> Result<T, Denial>,
) -> Result<T, Denial> {
    let budget = match (execution.is_leased(), budget) {
        (true, Some(budget)) => budget,
        _ => return operation(),
    };
    let mut result = None;
    let handoff = budget.run_stage(*execution, || result = Some(operation()));
    match (handoff, result) {
        (Ok(()), Some(result)) => result,
        (Ok(()), None) => Err(Denial::EncodingInconsistency),
        (Err(cause), result) => Err(final_denial(
            result.and_then(Result::err),
            Denial::Execution(cause.into()),
        )),
    }
}
