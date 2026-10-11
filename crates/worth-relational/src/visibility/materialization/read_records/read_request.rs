use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};

pub(crate) fn consult(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), Cause> {
    execution.consult().map_err(cause)
}

pub(super) fn cause(denial: worth_execution::WorkCeilingDenial) -> Cause {
    match crate::runtime::QueryReadExecutionStop::Request(denial).execution_denial() {
        Some(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            ..
        }) => cause,
        None => unreachable!("a request stop always carries its execution cause"),
    }
}

pub(super) fn checked<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: impl FnOnce() -> T,
) -> Result<T, Cause> {
    consult(execution)?;
    Ok(read())
}
