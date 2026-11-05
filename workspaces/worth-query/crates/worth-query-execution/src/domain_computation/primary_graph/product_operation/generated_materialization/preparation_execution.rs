//! One materialization preparation execution projection preserves the Native cause.
use crate::domain_computation::primary_graph::provider::relational_execution_denial;
use worth_relational::facade::{
    branch::RelationalMaterializationError,
    transactions::{CommitExecutionDenialKind, TransactionCommitError},
};
use {
    crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
    crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
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
