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
    pub(super) runtime: RelationalPreparationRuntime,
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
        crate::authority::commit::preparation::probe_symbol_rounds(
            execution,
            self,
            &basis,
            |mut symbols| {
                transaction.with_preparation_overlay(|transaction| {
                    transaction.ensure_current_basis(self)?;
                    if let Some(prior) = &transaction.pending_symbols {
                        symbols.adopt(prior);
                    }
                    let selected =
                        crate::branch::SelectedRelationalBranchState::from_admitted_basis(&basis);
                    transaction
                        .validate_staged_branch_locality(selected.state(), symbols.interner())?;
                    let input = RelationalTransactionValidationInput::from_transaction(transaction);
                    let identity =
                        self.issue_mutation_proposal_identity(transaction.transaction_id, &input)?;
                    transaction.reserve_preparation_client_keys(
                        &mut symbols,
                        self.config.identity.client_key_symbol_policy,
                        execution.allocation_policy(),
                    )?;
                    let runtime = self.with_candidate_symbols(&symbols);
                    let prepared = match lowered {
                        Some((selected, plan)) => prepare_lowered_working_state_scope(
                            &runtime,
                            transaction,
                            selected.clone(),
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
                        &prepared.selected_branch_state,
                        &prepared.working_state,
                        &prepared.merged_plan,
                        prepared.schema_authority.as_ref(),
                        &identity,
                        symbols,
                    )?;
                    let runtime = self.with_candidate_symbols(mutation.candidate_symbols());
                    Ok(PreparedProposalRound {
                        prepared,
                        mutation,
                        identity,
                        validation_input,
                        runtime,
                    })
                })
            },
        )
    }
}
