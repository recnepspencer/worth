//! One materialization preparation execution projection preserves the Native cause.
use crate::domain_computation::primary_graph::provider::relational_execution_denial;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind, WorthQueryProviderSessionDenialKind,
};
use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{CommitExecutionDenialKind, TransactionCommitError},
};

pub(super) fn project(
    error: &RelationalMaterializationError,
) -> Option<Result<WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionControlStopKind>> {
    let (cause, partition) = match error {
        RelationalMaterializationError::Read(cause) => (*cause, None),
        RelationalMaterializationError::Commit(TransactionCommitError::Execution {
            denial,
            ..
        }) => {
            // Exhaustive owner projection: a new Native denial kind needs a decision.
            let CommitExecutionDenialKind::Cause(cause) = denial.kind;
            (cause, denial.partition_identity)
        }
        _ => return None,
    };
    Some(relational_execution_denial::relational_execution_kind(
        cause, partition,
    ))
}
