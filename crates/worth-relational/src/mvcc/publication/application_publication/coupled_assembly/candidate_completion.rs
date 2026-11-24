//! Exact participant completion is sealed before the one final invariant judgment.
use super::*;
use crate::authority::mutation::MutationApplyDenial;
use crate::branch::selected_read::SelectedRelationalBranchRead;

pub(crate) fn prepare_coupled_candidate(
    execution: &ExecutionRequest<'_, '_>,
    round: &RelationalPreparationRuntime,
    transaction: &mut BranchBoundRelationalTransaction,
    selected: SelectedRelationalBranchRead<'_>,
    identity: &RelationalMutationProposalIdentity,
    symbols: &mut crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings,
    assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
) -> Result<bool, MutationApplyDenial> {
    let Some(binding) = transaction.authored_coupling.clone() else {
        return Ok(true);
    };
    let runtime = round.with_candidate_symbols(symbols);
    let mut prepared = crate::authority::commit::phases::prepare::prepare_working_state_scope(
        &runtime,
        transaction,
        selected,
        execution,
    )?;
    let authored_commands = transaction
        .prepared_coupled_commands
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .cloned()
        .expect("the ordinary merger carried the authored prefix");
    let owned_symbols = std::mem::replace(symbols, round.services.symbols.begin_pending_meanings());
    let mut mutation =
        crate::authority::commit::phases::applied_mutation::ordinary_apply::apply_authored_prefix(
            execution,
            &runtime,
            &selected,
            &prepared,
            authored_commands.plan(),
            identity,
            owned_symbols,
            &transaction.content,
        )?;
    if let Some(restoration) = binding.owners.source.restoration() {
        execution
            .run_native_with_checkpoint(|context| {
                crate::history::data::validate_authored_restoration_candidate(
                    &binding.owners.before,
                    &mutation.state,
                    &mutation.outcome.created_entities,
                    &mutation.outcome.created_relations,
                    restoration,
                    crate::authority::commit::pipeline::request_lease(execution),
                    context,
                )
            })
            .map_err(custody::execution_stop)?
            .0?;
    }
    let authored_records_memory = crate::mvcc::transaction::normalized_plan::reserve_copy(
        execution,
        crate::mvcc::transaction::normalized_plan::slots::<
            crate::transactions::data::primitives::RecordRef,
        >(mutation.outcome.effect.publication.changed_records.len()),
    )?;
    let mut authored_records = mutation.outcome.effect.publication.changed_records.clone();
    let authored_prefix_records = authored_records.len();
    let authored_prefix = 0..mutation.outcome.effect.publication.patch_fragments.len();
    writer_custody::require(
        execution,
        &binding.owners,
        writer_custody::Writer::AuthoredPrefix,
        authored_commands.plan(),
    )?;
    let mut fresh_records = net_plan::CreatorCustody::default();
    fresh_records.retain(
        execution,
        crate::publication::data::CoupledFreshWriter::Authored,
        binding.owners.source.subject(),
        &mutation.outcome,
    )?;
    let mut authored_writes = authored_writes::AuthoredWrites::default();
    authored_writes
        .retain(execution, 0, authored_commands.plan(), &authored_records)
        .map_err(|failure| failure.cause)?;
    let mut candidate = RelationalAuthoredCoupledCandidate {
        input_issuance: std::cell::RefCell::new(None),
        runtime: &runtime,
        transaction,
        prepared: &mut prepared,
        mutation: &mut mutation,
        identity,
        binding: &binding,
        parts: Vec::new(),
        stages: Vec::new(),
        authored_prefix,
        continuation: continuation::ContinuationState::default(),
        fresh_records,
        authored_writes,
        read: std::cell::RefCell::new(None),
        successor_rounds: 0,
    };
    crate::execution::phase_observation::symbol_preparation_event(
        crate::execution::phase_observation::SymbolPreparationEvent::AssemblerCallback,
    );
    if assembly
        .assemble(execution, &mut candidate)
        .map_err(|failure| failure.cause)?
        .is_break()
    {
        crate::execution::phase_observation::symbol_preparation_event(
            crate::execution::phase_observation::SymbolPreparationEvent::AssemblerBreak,
        );
        return Ok(false);
    }
    let RelationalAuthoredCoupledCandidate {
        mut parts,
        mut stages,
        authored_prefix,
        ..
    } = candidate;
    // The envelope's one naming plan: once, after the last member appended.
    let stage_plans = stages.iter_mut().map(|stage| stage.naming.take());
    let part_plans = parts.iter_mut().map(|part| part.naming.take());
    let root = transaction.naming_plan.clone();
    if let Some(plan) =
        super::part_naming::compose(execution, root.as_deref(), stage_plans.chain(part_plans))?
    {
        transaction.attach_naming_plan(plan);
    }
    let source_effects = roster_completion::complete(
        execution,
        &binding.owners,
        &parts,
        &stages,
        &mutation.outcome.effect.publication.patch_fragments,
        authored_prefix,
        &mut authored_records,
    )?;
    if let Some(batches) =
        super::succession::batches_after_withdrawal(execution, transaction.batches(), &parts)?
    {
        transaction
            .restage_coupled_field_batches(batches, execution.allocation_policy())
            .map_err(|cause| TransactionCommitError::conflict(cause.into_conflict()))?;
    }
    if !parts.is_empty() || !stages.is_empty() {
        execution
            .run_native_with_checkpoint(|context| {
                if let Some(composed) = super::disjoint_field_patches::compose(
                    execution,
                    context,
                    transaction.batches(),
                    &mut mutation.outcome.effect,
                    &mutation.state,
                    prepared.schema_authority.aspect_plans(),
                )? {
                    transaction
                        .restage_coupled_field_batches(
                            composed.batches,
                            execution.allocation_policy(),
                        )
                        .map_err(|stop| TransactionCommitError::conflict(stop.into_conflict()))?;
                }
                Ok::<(), TransactionCommitError>(())
            })
            .map_err(custody::execution_stop)?
            .0?;
    }
    let final_runtime = round.with_candidate_symbols(mutation.candidate_symbols());
    let intents = transaction.normalized_intents_for_merge(&final_runtime, execution)?;
    let commands =
        transaction.build_merged_plan_for_state(&final_runtime, &selected, intents, execution)?;
    transaction
        .footprint
        .derive_validation_dependencies(commands.plan(), execution.allocation_policy())
        .map_err(|failure| TransactionCommitError::conflict(failure.into_conflict()))?;
    mutation.aggregate = crate::aggregate_identity::AggregateProposal::capture(
        &prepared.schema_authority,
        selected.selected(),
        identity,
        &mutation.outcome.effect.publication.changed_records,
        &mutation.outcome.effect.adjacency.deltas,
        mutation.outcome.effect.publication.has_entity_deletions,
        None,
    );
    mutation.coupled_publication = Some(RequiredOutputPublicationAdmission {
        inputs: Default::default(),
        installed: binding.installed.clone(),
        publication: RequiredOutputPublication::CoupledAuthored {
            group: RelationalCoupledAuthoredPublication {
                binding: Arc::clone(&binding),
                source_effects,
                authored_records,
                authored_prefix_records,
                authored_commands,
                parts,
                stages,
                _authored_records_memory: authored_records_memory,
            },
        },
    });
    // The ordinary call below receives the complete actual symbol reservation
    // holder. Its adoption replaces this empty placeholder without cloning it.
    std::mem::swap(symbols, &mut mutation.identities.symbols);
    *binding.applied.lock().unwrap_or_else(|e| e.into_inner()) = Some(CoupledAppliedCandidate {
        identity: identity.clone(),
        commands,
        mutation,
    });
    Ok(true)
}
