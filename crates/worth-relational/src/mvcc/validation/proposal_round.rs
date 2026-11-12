//! Own one applied proposal round while the transaction keeps unique authority.
use super::proposal_source::RelationalProposalSource;
use crate::authority::commit::phases::applied_mutation::AppliedDraftCustody;
use crate::authority::commit::phases::prepare::PreparedWorkingStateScope;
use crate::authority::commit::pipeline::{CommitPhaseBacking, CommitPhaseReservation};
use crate::mvcc::{
    BranchBoundRelationalTransaction, RelationalMutationProposalIdentity,
    RelationalTransactionValidationInput,
};
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;

pub(super) struct PreparedProposalRound {
    pub(super) prepared: PreparedWorkingStateScope,
    pub(super) mutation: AppliedDraftCustody,
    pub(super) identity: RelationalMutationProposalIdentity,
    pub(super) validation_input: RelationalTransactionValidationInput,
    pub(super) invariants: crate::validation::engine::InvariantExecutionResult,
}

impl RelationalPreparationRuntime {
    pub(super) fn prepare_proposal_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: &mut BranchBoundRelationalTransaction,
        source: &RelationalProposalSource,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<CommitPhaseBacking<PreparedProposalRound>, TransactionCommitError> {
        let _cleanup =
            crate::authority::commit::phases::applied_mutation::CoupledPreparationCleanup::new(
                transaction.authored_coupling.as_ref(),
            );
        let basis = transaction.basis.clone();
        let selected = crate::branch::SelectedRelationalBranchState::from_admitted_basis(&basis);
        let (mut answer, round) = crate::storage::residency::probe_head(
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
            crate::execution::transaction_stage(
                execution,
                self.commit_work_budget.as_ref(),
                || {
                    self.mirror_residency
                        .demand_head(
                            execution,
                            &self.services.symbols,
                            basis.identity().branch_id(),
                            prepared
                                .value()
                                .prepared
                                .structural_summary
                                .touched_partitions
                                .iter()
                                .copied(),
                        )
                        .map_err(|failure| {
                            crate::authority::commit::preparation::storage_read_denial(
                                &basis, failure,
                            )
                        })
                },
            )?;
        }
        self.settle_preparation_round(&round);
        if let Ok(prepared) = &mut answer {
            prepared.value_mut().mutation.accept_probes();
        }
        answer
    }

    fn prepare_proposal_symbol_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: &mut BranchBoundRelationalTransaction,
        source: &RelationalProposalSource,
        selected: crate::branch::SelectedRelationalBranchRead<'_>,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<CommitPhaseBacking<PreparedProposalRound>, TransactionCommitError> {
        let basis = transaction.basis.clone();
        crate::authority::commit::preparation::probe_symbol_rounds(
            execution,
            self,
            &basis,
            |symbols, round| {
                transaction.with_funded_preparation_overlay(execution, |transaction| {
                    let storage = CommitPhaseReservation::prepare(execution)?;
                    let candidate = super::proposal_candidate::prepare(
                        self,
                        execution,
                        transaction,
                        source,
                        selected,
                        context,
                        symbols,
                        round,
                    )?;
                    super::proposal_candidate::validate(
                        execution, selected, round, candidate, storage,
                    )
                })
            },
        )
    }
}
