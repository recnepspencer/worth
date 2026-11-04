//! Select current custody once in the commit's original work account.
use crate::{
    branch::AdmittedRelationalBranchBasis,
    publication::ownership::{self, CurrentPublicationRoster},
    runtime::RelationalPreparationRuntime,
    transactions::data::TransactionCommitError,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
) -> Result<Option<Arc<CurrentPublicationRoster>>, TransactionCommitError> {
    if basis
        .inner
        .root
        .schema_authority()
        .registry()
        .current_publication_storage
        .is_none()
    {
        return Ok(None);
    }
    let run = || ownership::select_current(execution, basis);
    let result = match (
        super::super::super::request_lease(execution),
        runtime.commit_work_budget.as_ref(),
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, run)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?,
        _ => run(),
    }?;
    Ok(Some(result))
}
