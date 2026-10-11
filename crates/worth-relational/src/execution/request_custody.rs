use super::RelationalExecutionDenialCause as Cause;

pub(crate) fn check_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Cause> {
    request.check_live().map_err(super::denial::kernel_stop)
}

/// Checks caller interruption before a commit seam's first fallible reader.
pub(crate) fn check_commit_request(
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    check_request(request).map_err(|cause| {
        crate::transactions::data::TransactionCommitError::execution(
            crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            },
        )
    })
}
