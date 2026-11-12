//! Exact participant completion is sealed before the one final invariant judgment.
use super::*;
use crate::authority::mutation::MutationApplyDenial;
use crate::branch::SelectedRelationalBranchRead;
use crate::execution::request_stage::checkpoint;
use crate::publication::data::AuthoredRetirementRefusal as Cause;

/// The Native-applied group retains each writer's concrete admission and commands.
/// Its constructor is private; declarations and submitted rosters cannot mint it.
pub struct RelationalCoupledAuthoredPublication {
    pub(crate) binding: Arc<BoundAuthoredCoupling>,
    pub(crate) source_effects: RelationalCoupledAuthoredSourceEffects,
    pub(crate) authored_records: Vec<crate::transactions::data::RecordRef>,
    pub(crate) authored_commands: Arc<crate::mvcc::OwnedNormalizedPlan>,
    pub(crate) parts: Vec<CoupledPublicationPart>,
    pub(crate) stages: Vec<CoupledAuthoredStagePart>,
    _authored_records_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl std::fmt::Debug for RelationalCoupledAuthoredPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalCoupledAuthoredPublication")
            .field("owners", &self.binding.owners)
            .field("parts", &self.parts.len())
            .finish_non_exhaustive()
    }
}
impl RelationalCoupledAuthoredPublication {
    pub(crate) fn admitted_by(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
    ) -> bool {
        self.binding.installed.runtime_instance_id == installed.runtime_instance_id
            && self
                .parts
                .iter()
                .all(|part| part.publication.admitted_by(installed))
            && self.binding.retirement_admission.as_ref().map_or(
                self.binding.owners.retirements.is_empty(),
                |admission| {
                    self.binding.owners.retirements.iter().all(|prior| {
                        installed
                            .authority
                            .admits_authored_output_retirement(admission, prior.authority())
                    })
                },
            )
    }
}

/// The applied image is held only until the existing ordinary apply phase
/// adopts it. It is neither a preview nor a second application of its commands.
pub(crate) struct CoupledAppliedCandidate {
    pub(crate) identity: RelationalMutationProposalIdentity,
    pub(crate) commands: crate::mvcc::OwnedNormalizedPlan,
    pub(crate) mutation: AppliedMutationCore,
}

pub(crate) fn prepare_coupled_candidate(
    execution: &ExecutionRequest<'_, '_>,
    round: &RelationalPreparationRuntime,
    transaction: &mut BranchBoundRelationalTransaction,
    selected: SelectedRelationalBranchRead<'_>,
    identity: &RelationalMutationProposalIdentity,
    symbols: &mut crate::runtime::PendingSymbolMeanings,
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
    let authored_records_memory = crate::mvcc::plan_copy_admission::reserve_copy(
        execution,
        crate::mvcc::plan_copy_admission::slots::<crate::transactions::data::RecordRef>(
            mutation.outcome.effect.publication.changed_records.len(),
        ),
    )?;
    let mut authored_records = mutation.outcome.effect.publication.changed_records.clone();
    let authored_fragments = mutation.outcome.effect.publication.patch_fragments.len();
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
        runtime: &runtime,
        transaction,
        prepared: &mut prepared,
        mutation: &mut mutation,
        identity,
        binding: &binding,
        parts: Vec::new(),
        stages: Vec::new(),
        authored_fragments,
        continuation: continuation::ContinuationState::default(),
        fresh_records,
        authored_writes,
        read: std::cell::RefCell::new(None),
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
        parts,
        stages,
        authored_fragments,
        ..
    } = candidate;
    if let Some(missing) = binding.owners.stages.get(stages.len()) {
        return Err(part::reject(Cause::MissingAuthoredStage, *missing).into());
    }
    // Every authored stage applies before any producer part, so the authored
    // writer's patches are exactly this applied prefix of the candidate's.
    let source_effects = RelationalCoupledAuthoredSourceEffects::capture(
        execution,
        &mutation.outcome.effect.publication.patch_fragments[..authored_fragments],
    )?;
    for stage in &stages {
        authored_records.extend_from_slice(&stage.records);
    }
    execution.run_native_with_checkpoint(|context| {
        for expected in binding.owners.required_owners() {
            for part in &parts {
                let meaning = part.publication.producing_meaning().expect("admitted producer route");
                checkpoint(context, (meaning.family.len() + expected.origin.meaning.family.len() + 65) as u64)
                    .map_err(custody::execution_stop)?;
            }
            if !parts.iter().any(|part| matches!(part.publication.owned(),
                Some(RequiredOutputPublication::Preserve { meaning, .. }) if expected.origin.meaning.same_route(meaning))) {
                return Err(part::reject(Cause::MissingCoupledOwner, expected.origin.meaning.source_scope));
            }
        }
        Ok(())
    }).map_err(custody::execution_stop)?.0?;
    if !parts.is_empty() || !stages.is_empty() {
        super::disjoint_field_patches::compose(
            execution,
            transaction,
            &mut mutation.outcome.effect,
            &mutation.state,
            prepared.schema_authority.aspect_plans(),
        )?;
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
        installed: binding.installed.clone(),
        publication: RequiredOutputPublication::CoupledAuthored {
            group: RelationalCoupledAuthoredPublication {
                binding: Arc::clone(&binding),
                source_effects,
                authored_records,
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
