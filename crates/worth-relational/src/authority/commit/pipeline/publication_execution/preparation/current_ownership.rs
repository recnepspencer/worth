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
    candidate: &crate::history::data::CanonicalCommitEnvelope,
    admission: Option<&crate::mvcc::RequiredOutputPublicationAdmission>,
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
    let mut records = candidate
        .patch
        .authoritative_record_patches
        .iter()
        .filter(|p| {
            p.structural_change != crate::publication::patch::data::RecordStructuralChange::Created
        })
        .map(|p| p.target.clone())
        .collect::<Vec<_>>();
    if let Some(admission) = admission {
        admission.publication.ownership_seeds(&mut records);
    }
    for event in &candidate.required_output_codec_retirements {
        records.push(crate::transactions::data::RecordRef::Entity(
            event.meaning.source_scope,
        ));
    }
    if let Some(current) = admission.and_then(|a| a.publication.current_ownership_evidence())
        && current.basis == *basis.descriptor()
        && current.covers(&records)
    {
        return Ok(Some(Arc::clone(current)));
    }
    let run = || ownership::select_current(execution, basis, &records);
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
