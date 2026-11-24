use super::*;

pub(super) fn cause(error: TransactionCommitError) -> Cause {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("original execution cause required: {error:?}")
    };
    let crate::transactions::data::CommitExecutionDenialKind::Cause(cause) = denial.kind;
    cause
}
