//! Own one applied proposal round while the transaction keeps unique authority.
use super::proposal_source::RelationalProposalSource;
use crate::authority::commit::phases::applied_mutation::{
    apply_mutation_draft, apply_retained_entity_selection_draft, AppliedMutationDraft,
};
use crate::authority::commit::phases::prepare::{
    prepare_lowered_working_state_scope, prepare_working_state_scope, PreparedWorkingStateScope,
};
use crate::mvcc::{
    BranchBoundRelationalTransaction, RelationalMutationProposalIdentity,
    RelationalTransactionValidationInput,
};
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;

pub(super) struct PreparedProposalRound {
    pub(super) prepared: PreparedWorkingStateScope,
    pub(super) mutation: AppliedMutationDraft,
    pub(super) identity: RelationalMutationProposalIdentity,
    pub(super) validation_input: RelationalTransactionValidationInput,
    pub(super) coupled_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    pub(super) invariants: crate::validation::engine::InvariantExecutionResult,
}

impl RelationalPreparationRuntime {
    pub(super) fn prepare_proposal_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: &mut BranchBoundRelationalTransaction,
        source: &RelationalProposalSource,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<PreparedProposalRound, TransactionCommitError> {
        let basis = transaction.basis.clone();
        let selected = crate::branch::SelectedRelationalBranchState::from_admitted_basis(&basis);
        let (answer, round) = crate::storage::residency::probe_head(
            execution,
            &selected.with_symbol_admission(&self.services.symbols),
            |partitions| {
                let round = self
                    .preparation_round()
                    .with_authored_coupling(transaction.authored_coupling.clone());
                let read =
                    crate::branch::SelectedRelationalBranchRead::in_round(&selected, partitions);
                let answer = round.prepare_proposal_symbol_round(
                    execution,
                    transaction,
                    source,
                    read,
                    context,
                );
                (answer, round)
            },
        )
        .map_err(|failure| {
            crate::authority::commit::preparation::storage_read_denial(&basis, failure)
        })?;
        if let Ok(prepared) = &answer {
            self.mirror_residency
                .demand_head(
                    execution,
                    &self.services.symbols,
                    basis.identity().branch_id(),
                    prepared
                        .prepared
                        .structural_summary
                        .touched_partitions
                        .iter()
                        .copied(),
                )
                .map_err(|failure| {
                    crate::authority::commit::preparation::storage_read_denial(&basis, failure)
                })?;
        }
        self.settle_preparation_round(&round);
        answer
    }

    fn prepare_proposal_symbol_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: &mut BranchBoundRelationalTransaction,
        source: &RelationalProposalSource,
        selected: crate::branch::SelectedRelationalBranchRead<'_>,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<PreparedProposalRound, TransactionCommitError> {
        let basis = transaction.basis.clone();
        crate::authority::commit::preparation::probe_symbol_rounds(
            execution,
            self,
            &basis,
            |mut symbols, round| {
                transaction.with_preparation_overlay(|transaction| {
                    transaction.ensure_current_basis(self)?;
                    if let Some(prior) = &transaction.pending_symbols {
                        symbols.adopt(prior)?;
                    }
                    if let Some(binding) = &transaction.authored_coupling {
                        binding.inherit_prepared_symbols(&mut symbols)?;
                    }
                    transaction.validate_staged_branch_locality(&selected, symbols.interner())?;
                    let input = RelationalTransactionValidationInput::from_transaction(transaction);
                    let (identity, source_interpretation) =
                        self.issue_mutation_proposal_identity(transaction.transaction_id, &input)?;
                    source_interpretation.bind_context(execution, context)?;
                    transaction.reserve_preparation_client_keys(
                        &mut symbols,
                        self.config.identity.client_key_symbol_policy,
                        execution.allocation_policy(),
                    )?;
                    if let Some(binding) = &transaction.authored_coupling {
                        binding.adopt_prepared_symbols(&identity, &mut symbols)?;
                    }
                    let runtime = round.with_candidate_symbols(&symbols);
                    let prepared = match source {
                        RelationalProposalSource::Transaction { lowered: Some((_, plan)), .. } => prepare_lowered_working_state_scope(
                            &runtime,
                            transaction,
                            selected,
                            plan.clone(),
                        ),
                        RelationalProposalSource::Transaction { lowered: None, .. } => prepare_working_state_scope(
        execution,
                            &runtime,
                            transaction,
                            selected,
                            execution.allocation_policy(),
                        )?,
                        RelationalProposalSource::RetainedEntitySelection(retained) => {
                            crate::authority::commit::phases::prepare::retained_entity_selection::prepare(
                                &runtime, transaction, selected, retained, execution.allocation_policy(),
                            )?
                        }
                    };
                    let validation_input =
                        RelationalTransactionValidationInput::from_transaction(transaction);
                    let mut mutation = match prepared.plan.retained() {
                        Some(retained) => apply_retained_entity_selection_draft(
                            execution, &runtime, &selected, &prepared.working_state, retained,
                            prepared.schema_authority.as_ref(), &identity, symbols,
                            source_interpretation, &validation_input.content,
                        ),
                        None => apply_mutation_draft(
                        execution,
                        &runtime,
                        &selected,
                        &prepared.working_state,
                        prepared.plan.view().commands(),
                        prepared.schema_authority.as_ref(),
                        &identity,
                        symbols,
                        source_interpretation,
                        &validation_input.content,
                    ),
                    }?;
                    let runtime = round.with_candidate_symbols(mutation.candidate_symbols());
                    mutation.prepare_managed_sources(
                        *execution,
                        &runtime,
                        selected,
                        &identity,
                        (validation_input.merge_parent_bases().is_empty()
                            && prepared.plan.retained().is_none())
                            .then_some(prepared.plan.view().commands()),
                    )?;
                    let control = crate::validation::engine::InvariantPreparationControl::for_transaction(
                        &validation_input, execution.allocation_policy(), crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
                    );
                    let pending = runtime.invariant_authority().begin_proposed_state(
                        &selected, mutation.state(), identity.proposed_version_id(), prepared.plan.view(), Some(&identity),
                        mutation.aggregate_source(selected), execution, &control,
                    )?;
                    mutation.prepare_aggregate(
                        execution,
                        round,
                        selected,
                        prepared.plan.view().commands(),
                    )?;
                    let aggregate_admission = mutation.complete_aggregate_draft(execution, &runtime, &selected, prepared.plan.view(), &validation_input, &identity)?;
                    let runtime = round.with_candidate_symbols(mutation.candidate_symbols());
                    let invariant_plan = aggregate_admission.as_ref().map_or(prepared.plan.view(), |admission| prepared.plan.view().with_commands(&admission.plan));
                    let invariants = runtime.invariant_authority().enforce_proposed_state(
                        &selected, mutation.state(), pending, &mutation.outcome.created_entities,
                        identity.proposed_version_id(), invariant_plan, Some(&identity),
                        mutation.aggregate_source(selected), execution, &control,
                    )?;
                    Ok(PreparedProposalRound {
                        prepared,
                        mutation,
                        identity,
                        validation_input,
                        coupled_publication: transaction.authored_coupling.as_ref().and_then(|binding| {
                            binding.completed.lock().unwrap_or_else(|e| e.into_inner()).take()
                        }),
                        invariants,
                    })
                })
            },
        )
    }
}
