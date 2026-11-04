//! Complete current ownership inside the original commit publication preparation.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn complete(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    selected: &crate::branch::SelectedRelationalBranchState,
    working: &mut crate::storage::overlay::WorkingState,
    identities: &mut crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    canonical: &mut crate::history::data::CanonicalCommitEnvelope,
    artifacts: &mut crate::storage::overlay::PublicationArtifacts,
    changed: &mut Vec<crate::transactions::data::RecordRef>,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, TransactionCommitError> {
    let Some(current) = &runtime.current_ownership else {
        return Ok(None);
    };
    let mut answer = None;
    let mut run = || {
        let handoff = execution.run_native_with_checkpoint(|context| {
            answer = Some(crate::publication::ownership::complete(
                execution, runtime, basis, selected, working, identities, canonical, current,
                context,
            ));
        });
        match (handoff, answer.take()) {
            (Ok(_), Some(answer)) => answer,
            (Err(_), Some(Err(cause))) => Err(cause),
            (Err(cause), _) => Err(TransactionCommitError::execution(cause.into())),
            (Ok(_), None) => unreachable!("completed publication ownership stage executed"),
        }
    };
    let completed = match (
        super::super::super::request_lease(execution),
        runtime.commit_work_budget.as_ref(),
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, run)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?,
        _ => run(),
    }?;
    let mut effect = completed.effect;
    execution
        .in_scope(|lease| {
            crate::authority::commit::publication::extend_patch(
                runtime,
                &mut canonical.patch,
                std::mem::take(&mut effect.publication.patch_fragments),
                lease,
            )
        })
        .map_err(|cause| TransactionCommitError::execution(cause.into()))??;
    changed.extend(effect.publication.changed_records);
    crate::authority::commit::phases::publication::canonicalize_changed_records(changed);
    canonical.install_record_allocations(identities.records.canonical().to_vec());
    identities.records.finish_mutation().map_err(|denial| {
        TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
            crate::transactions::data::ConflictClass::RecordAllocationDenied { denial },
        ))
    })?;
    canonical.revision_symbol_references = identities
        .symbols
        .required_references()
        .iter()
        .copied()
        .collect();
    canonical.symbol_texts = runtime
        .services
        .symbols
        .with_read(|symbols| {
            symbols.commit_texts(
                &canonical.merged_plan,
                &canonical.patch,
                canonical.revision_symbol_references.iter().copied(),
            )
        })
        .map_err(crate::mvcc::origin_denial)?;
    artifacts.patch = canonical.patch.clone();
    Ok(Some(completed.funding))
}
