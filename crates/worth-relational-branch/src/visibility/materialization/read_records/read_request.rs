use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial;
use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;

pub(super) fn cause(denial: worth_execution::WorkCeilingDenial) -> Cause {
    match crate::visibility::materialization::read_records::reader::query_execution::leased_map::QueryReadExecutionStop::Request(denial).execution_denial() {
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

pub fn checked<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: impl FnOnce() -> T,
) -> Result<T, Cause> {
    crate::execution::request_custody::check_request(execution)?;
    Ok(read())
}
