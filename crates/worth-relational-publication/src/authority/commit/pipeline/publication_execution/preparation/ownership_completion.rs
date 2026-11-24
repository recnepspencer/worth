//! Complete current ownership inside the original commit publication preparation.
use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelopeBodyFieldsPort as _, CanonicalCommitEnvelopeBodyOwnerPort as _,
};
use worth_relational_branch::internal_port::publication::ownership::read::roster::CurrentPublicationRoster;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub(super) struct OwnershipCompletionInputs<'a> {
    pub(super) current: Option<&'a CurrentPublicationRoster>,
    pub(super) storage: &'a contracts::config::data::sections::StorageConfig,
    pub(super) symbols: &'a RuntimeSymbolTable,
    pub(super) diagnostics: PreparationDiagnostics<'a>,
    pub(super) work: Option<&'a contracts::execution::request_work::RequestWorkBudget>,
    pub(super) max_patch_records: Option<usize>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn complete(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: OwnershipCompletionInputs<'_>,
    basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    selected: &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    working: &mut worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    identities: &mut worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
    canonical: &mut worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft,
    artifacts: &mut crate::publication::authority::artifacts::PublicationArtifacts,
    changed: &mut Vec<contracts::transactions::data::primitives::RecordRef>,
    positions: &mut Option<worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputPublicationPositions>,
) -> Result<
    (
        Option<(
            worth_execution::ExecutionMemoryReservation,
            worth_execution::ExecutionMemoryReservation,
        )>,
        Vec<worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::data::CanonicalRecordAspectDelta>,
        usize,
    ),
    TransactionCommitError,
>{
    let Some(current) = inputs.current else {
        return Ok((None, Vec::new(), 0));
    };
    let mut answer = None;
    let mut run = || {
        let handoff = execution.run_native_with_checkpoint(|context| {
            answer = Some((|| {
                let mut position =
                    worth_relational_mutation::internal_port::history::data::required_output_positions::presence::completion::complete(
                        execution,
                        inputs.storage,
                        basis,
                        working,
                        identities,
                        canonical,
                        &current.positions,
                        positions,
                        context,
                    )?;
                let roster = crate::publication::ownership::complete(
                    execution,
                    inputs.storage,
                    basis,
                    selected,
                    working,
                    identities,
                    canonical,
                    current,
                    context,
                )?;
                position.effect.accumulate(roster.effect);
                let count = position.effect.adjacency.deltas.len();
                let prior = match canonical.descriptive_touches() {
                    worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouchGraph::Exact(touches)
                        if count != 0 =>
                    {
                        touches.len()
                    }
                    _ => 0,
                };
                let overflow = || {
                    contracts::publication::ownership::read_request::request(
                        contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
                    )
                };
                let total = if count == 0 {
                    0
                } else {
                    prior
                        .checked_add(count.checked_mul(4).ok_or_else(overflow)?)
                        .ok_or_else(overflow)?
                };
                let work = total
                    .checked_mul((usize::BITS - total.leading_zeros()) as usize + 2)
                    .ok_or_else(overflow)?;
                contracts::publication::ownership::read_request::checkpoint(
                    context,
                    u64::try_from(work).map_err(|_| overflow())?,
                )?;
                // Fund vector growth and the sort's scratch before extending the
                // already retained transcript. Empty private effects do no work.
                let bytes = (total as u64)
                    .checked_mul(
                        (4 * std::mem::size_of::<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::descriptive_touches::RelationalDescriptiveTouch>(
                        )) as u64,
                    )
                    .and_then(|extra| position.funding.bytes().checked_add(extra))
                    .ok_or_else(overflow)?;
                position.funding.resize(bytes).map_err(|cause| {
                    contracts::publication::ownership::read_request::request(contracts::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })?;
                canonical.complete_native_adjacency(&position.effect.adjacency.deltas);
                // Both original-account reservations live through the canonical handoff.
                Ok::<_, TransactionCommitError>((position, roster.funding))
            })());
        });
        match (handoff, answer.take()) {
            (Ok(_), Some(answer)) => answer,
            (Err(_), Some(Err(cause))) => Err(cause),
            (Err(cause), _) => Err(TransactionCommitError::execution(cause.into())),
            (Ok(_), None) => unreachable!("completed publication ownership stage executed"),
        }
    };
    let (completed, roster_funding) = match (
        contracts::execution::request_custody::request_lease(execution),
        inputs.work,
    ) {
        (Some(_), Some(budget)) => budget
            .run_stage(*execution, run)
            .map_err(|cause| TransactionCommitError::execution(cause.into()))?,
        _ => run(),
    }?;
    let mut effect = completed.effect;
    let fragments = std::mem::take(&mut effect.publication.patch_fragments);
    execution
        .in_scope(|lease| {
            if fragments.is_empty() {
                return Ok(());
            }
            crate::authority::commit::publication::extend_patch(
                inputs.diagnostics.instrumentation,
                inputs.work,
                &mut canonical.patch,
                fragments,
                lease,
            )?;
            crate::authority::commit::phases::publication::enforce_patch_budget(
                inputs.max_patch_records,
                inputs.diagnostics,
                &canonical.patch,
            )
        })
        .map_err(|cause| TransactionCommitError::execution(cause.into()))??;
    changed.extend(effect.publication.changed_records);
    crate::authority::commit::phases::publication::canonicalize_changed_records(changed);
    canonical.install_record_allocations(identities.records.canonical().to_vec());
    identities.records.finish_mutation().map_err(|denial| {
        TransactionCommitError::conflict(contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
            contracts::transactions::data::outcomes::conflict_class::ConflictClass::RecordAllocationDenied { denial },
        ))
    })?;
    (*canonical.owner_revision_symbol_references_mut()) = identities
        .symbols
        .required_references()
        .iter()
        .copied()
        .collect();
    (*canonical.owner_symbol_texts_mut()) = inputs
        .symbols
        .with_read(|symbols| {
            symbols.commit_texts(
                &canonical.merged_plan,
                &canonical.patch,
                (*canonical.owner_revision_symbol_references())
                    .iter()
                    .copied(),
            )
        })
        .map_err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial,
        )?;
    artifacts.patch = canonical.patch.clone();
    Ok((
        Some((completed.funding, roster_funding)),
        effect.publication.canonical_deltas,
        effect.adjacency.deltas.len(),
    ))
}
