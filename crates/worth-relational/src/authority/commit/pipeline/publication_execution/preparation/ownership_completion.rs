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
    canonical: &mut crate::history::data::CanonicalCommitEnvelopeDraft,
    artifacts: &mut crate::storage::overlay::PublicationArtifacts,
    changed: &mut Vec<crate::transactions::data::RecordRef>,
    positions: &mut Option<crate::history::data::RequiredOutputPublicationPositions>,
) -> Result<
    (
        Option<(
            worth_execution::ExecutionMemoryReservation,
            worth_execution::ExecutionMemoryReservation,
        )>,
        Vec<crate::authority::mutation::CanonicalRecordAspectDelta>,
        usize,
    ),
    TransactionCommitError,
> {
    let Some(current) = &runtime.current_ownership else {
        return Ok((None, Vec::new(), 0));
    };
    let mut answer = None;
    let mut run = || {
        let handoff = execution.run_native_with_checkpoint(|context| {
            answer = Some((|| {
                let mut position =
                    crate::history::data::current_route_presence::completion::complete(
                        execution,
                        runtime,
                        basis,
                        working,
                        identities,
                        canonical,
                        &current.positions,
                        positions,
                        context,
                    )?;
                let roster = crate::publication::ownership::complete(
                    execution, runtime, basis, selected, working, identities, canonical, current,
                    context,
                )?;
                position.effect.accumulate(roster.effect);
                let count = position.effect.adjacency.deltas.len();
                let prior = match canonical.descriptive_touches() {
                    crate::history::data::RelationalDescriptiveTouchGraph::Exact(touches)
                        if count != 0 =>
                    {
                        touches.len()
                    }
                    _ => 0,
                };
                let overflow = || {
                    crate::publication::ownership::request(
                        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
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
                crate::publication::ownership::checkpoint(
                    context,
                    u64::try_from(work).map_err(|_| overflow())?,
                )?;
                // Fund vector growth and the sort's scratch before extending the
                // already retained transcript. Empty private effects do no work.
                let bytes = (total as u64)
                    .checked_mul(
                        (4 * std::mem::size_of::<crate::history::data::RelationalDescriptiveTouch>(
                        )) as u64,
                    )
                    .and_then(|extra| position.funding.bytes().checked_add(extra))
                    .ok_or_else(overflow)?;
                position.funding.resize(bytes).map_err(|cause| {
                    crate::publication::ownership::request(crate::execution::denial::lease_denial(
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
    Ok((
        Some((completed.funding, roster_funding)),
        effect.publication.canonical_deltas,
        effect.adjacency.deltas.len(),
    ))
}
