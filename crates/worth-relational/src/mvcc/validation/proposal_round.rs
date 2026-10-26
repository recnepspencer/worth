//! Own one applied proposal round while the transaction keeps unique authority.
use crate::authority::commit::phases::applied_mutation::{
    apply_mutation_draft, AppliedMutationDraft,
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
    pub(super) commit_boundary: crate::validation::engine::InvariantExecutionResult,
    pub(super) mutation_sensitive: crate::validation::engine::InvariantExecutionResult,
    pub(super) publication: crate::validation::engine::InvariantExecutionResult,
}

impl RelationalPreparationRuntime {
    pub(super) fn prepare_proposal_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: &mut BranchBoundRelationalTransaction,
        lowered: Option<&(
            crate::branch::SelectedRelationalBranchState,
            crate::transactions::data::MergedCommitPlan,
        )>,
    ) -> Result<PreparedProposalRound, TransactionCommitError> {
        let basis = transaction.basis.clone();
        let selected = crate::branch::SelectedRelationalBranchState::from_admitted_basis(&basis);
        let (answer, round) =
            crate::storage::residency::probe_full(execution, &selected, |partitions| {
                let round = self.preparation_round();
                let read =
                    crate::branch::SelectedRelationalBranchRead::in_round(&selected, partitions);
                let answer =
                    round.prepare_proposal_symbol_round(execution, transaction, lowered, read);
                (answer, round)
            })
            .map_err(|failure| {
                crate::authority::commit::preparation::storage_read_denial(&basis, failure)
            })?;
        if let Ok(prepared) = &answer {
            self.mirror_residency
                .demand(
                    execution,
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
        lowered: Option<&(
            crate::branch::SelectedRelationalBranchState,
            crate::transactions::data::MergedCommitPlan,
        )>,
        selected: crate::branch::SelectedRelationalBranchRead<'_>,
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
                        symbols.adopt(prior);
                    }
                    transaction.validate_staged_branch_locality(&selected, symbols.interner())?;
                    let input = RelationalTransactionValidationInput::from_transaction(transaction);
                    let identity =
                        self.issue_mutation_proposal_identity(transaction.transaction_id, &input)?;
                    transaction.reserve_preparation_client_keys(
                        &mut symbols,
                        self.config.identity.client_key_symbol_policy,
                        execution.allocation_policy(),
                    )?;
                    let runtime = round.with_candidate_symbols(&symbols);
                    let prepared = match lowered {
                        Some((_, plan)) => prepare_lowered_working_state_scope(
                            &runtime,
                            transaction,
                            selected,
                            plan.clone(),
                        ),
                        None => prepare_working_state_scope(
                            &runtime,
                            transaction,
                            selected,
                            execution.allocation_policy(),
                        )?,
                    };
                    let validation_input =
                        RelationalTransactionValidationInput::from_transaction(transaction);
                    let mutation = apply_mutation_draft(
                        execution,
                        &runtime,
                        &selected,
                        &prepared.working_state,
                        &prepared.merged_plan,
                        prepared.schema_authority.as_ref(),
                        &identity,
                        symbols,
                    )?;
                    let runtime = round.with_candidate_symbols(mutation.candidate_symbols());
                    let control =
                        crate::validation::engine::InvariantPreparationControl::for_transaction(
                            &validation_input,
                            execution.allocation_policy(),
                            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
                        );
                    let commit_boundary = runtime
                        .invariant_authority()
                        .enforce_commit_boundary_for_selected_branch(
                            &selected,
                            mutation.state(),
                            identity.proposed_version_id(),
                            &prepared.merged_plan,
                            Some(&identity),
                            execution,
                            &control,
                        )?;
                    let (mutation_sensitive, publication) =
                        super::proposal_invariants::validate_proposed_state(
                            &runtime,
                            &prepared,
                            &selected,
                            mutation.state(),
                            identity.proposed_version_id(),
                            Some(&identity),
                            execution,
                            &control,
                        )?;
                    Ok(PreparedProposalRound {
                        prepared,
                        mutation,
                        identity,
                        validation_input,
                        commit_boundary,
                        mutation_sensitive,
                        publication,
                    })
                })
            },
        )
    }
}
