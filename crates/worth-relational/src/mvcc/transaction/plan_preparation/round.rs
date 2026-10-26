//! A plan preview abandons normalized input and telemetry before cold loads.
use super::super::DetachedRelationalTransactionOverlay;
use super::BranchBoundRelationalTransaction;
use crate::branch::{SelectedRelationalBranchRead, SelectedRelationalBranchState};
use crate::mvcc::RelationalTransactionFootprint;
use crate::runtime::{PendingSymbolMeanings, RelationalPreparationRuntime};
use crate::transactions::data::{MergedCommitPlan, TransactionCommitError};

pub(super) struct PreparedPlanRound {
    pub(super) plan: MergedCommitPlan,
    pub(super) symbols: PendingSymbolMeanings,
    pub(super) overlay: DetachedRelationalTransactionOverlay,
    pub(super) footprint: RelationalTransactionFootprint,
}

pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    preparation: &RelationalPreparationRuntime,
    transaction: &mut BranchBoundRelationalTransaction,
) -> Result<PreparedPlanRound, TransactionCommitError> {
    let basis = transaction.basis.clone();
    let selected = SelectedRelationalBranchState::from_admitted_basis(&basis);
    let allocation = execution.allocation_policy();
    let (answer, effects) = crate::storage::residency::probe_head(
        execution,
        &selected.with_symbol_admission(&preparation.services.symbols),
        |partitions| {
            let effects = preparation.preparation_round();
            let read = SelectedRelationalBranchRead::in_round(&selected, partitions);
            let answer = crate::authority::commit::preparation::probe_symbol_rounds(
                execution,
                &effects,
                &basis,
                |mut symbols, round| {
                    transaction.with_preparation_overlay(|transaction| {
                        if let Some(prior) = &transaction.pending_symbols {
                            symbols.adopt(prior);
                        }
                        transaction.validate_staged_branch_locality(&read, symbols.interner())?;
                        transaction.reserve_preparation_client_keys(
                            &mut symbols,
                            preparation.config.identity.client_key_symbol_policy,
                            allocation,
                        )?;
                        let runtime = round.with_candidate_symbols(&symbols);
                        let intents =
                            transaction.normalized_intents_for_merge(&runtime, allocation)?;
                        let plan =
                            transaction.build_merged_plan_for_state(&runtime, &read, intents)?;
                        Ok(PreparedPlanRound {
                            plan,
                            symbols,
                            overlay: transaction.overlay.clone(),
                            footprint: transaction.footprint.clone(),
                        })
                    })
                },
            );
            (answer, effects)
        },
    )
    .map_err(|failure| {
        crate::authority::commit::preparation::storage_read_denial(&basis, failure)
    })?;
    preparation.settle_preparation_round(&effects);
    answer
}
