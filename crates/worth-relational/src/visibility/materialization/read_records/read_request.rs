use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};

pub(super) fn cause(denial: worth_execution::WorkCeilingDenial) -> Cause {
    match crate::runtime::QueryReadExecutionStop::Request(denial).execution_denial() {
        Some(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            ..
        }) => cause,
        None => unreachable!(
            /* Proof: This private adapter receives only request-level stops; its producer always attaches an execution cause before constructing the stop. */
            "a request stop always carries its execution cause"
        ),
    }
}

pub(super) fn checked<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: impl FnOnce() -> T,
) -> Result<T, Cause> {
    crate::execution::check_request(execution)?;
    Ok(read())
}
