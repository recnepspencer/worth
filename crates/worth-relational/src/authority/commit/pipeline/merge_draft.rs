//! An independent merge abandons its entire applied draft on any read miss.
use super::authority_context::PreparedAuthorityScope;
use super::execution_admission::AdmittedCommitExecution;
use crate::authority::commit::phases::applied_mutation::apply_mutation_draft;
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::branch::SelectedRelationalBranchRead;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::{InvariantExecutionResult, InvariantPreparationControl};

pub(super) struct AcceptedMergeDraft {
    pub(super) scope: PreparedAuthorityScope,
    pub(super) boundary: InvariantExecutionResult,
    pub(super) mutation_sensitive: InvariantExecutionResult,
    pub(super) publication: InvariantExecutionResult,
}

pub(super) fn prepare_merge_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    admitted: &AdmittedCommitExecution,
) -> Result<AcceptedMergeDraft, TransactionCommitError> {
    let basis = admitted.validation_input().basis();
    let selected = admitted.selected_branch_state();
    crate::authority::commit::preparation::demand_mutation_inputs(
        execution,
        basis,
        &runtime.services.symbols,
        &admitted.merged_plan().merged_intents,
        Default::default(),
    )?;
    let (answer, round) = crate::storage::residency::probe_full(
        execution,
        &selected.with_symbol_admission(&runtime.services.symbols),
        |partitions| {
            let round = runtime.preparation_round();
            let read = SelectedRelationalBranchRead::in_round(selected, partitions);
            let answer = prepare_symbol_round(execution, &round, admitted, read);
            (answer, round)
        },
    )
    .map_err(|failure| {
        crate::authority::commit::preparation::storage_read_denial(basis, failure)
    })?;
    if let Ok(accepted) = &answer {
        runtime
            .mirror_residency
            .demand(
                execution,
                &runtime.services.symbols,
                basis.identity().branch_id(),
                accepted
                    .scope
                    .structural_summary
                    .touched_partitions
                    .iter()
                    .copied(),
            )
            .map_err(|failure| {
                crate::authority::commit::preparation::storage_read_denial(basis, failure)
            })?;
    }
    runtime.settle_preparation_round(&round);
    answer
}

fn prepare_symbol_round(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    admitted: &AdmittedCommitExecution,
    selected: SelectedRelationalBranchRead<'_>,
) -> Result<AcceptedMergeDraft, TransactionCommitError> {
    let input = admitted.validation_input();
    crate::authority::commit::preparation::probe_symbol_rounds(
        execution,
        runtime,
        input.basis(),
        |symbols, round| {
            let identity =
                round.issue_mutation_proposal_identity(admitted.transaction_id(), input)?;
            let parent_count = admitted
                .merge_history_plan()
                .map_or(input.merge_parent_bases().len(), |plan| {
                    plan.requested_merge_parent_count
                });
            let (structural_summary, working_state, phase_timing) =
                prepare_authoritative_working_state_scope_for_base(
                    round,
                    &selected,
                    admitted.merged_plan(),
                    parent_count,
                    Some(input.footprint()),
                );
            let mutation = apply_mutation_draft(
                execution,
                round,
                &selected,
                &working_state,
                admitted.merged_plan(),
                input.schema_authority(),
                &identity,
                symbols,
            )?;
            let candidate = round.with_candidate_symbols(mutation.candidate_symbols());
            let invariants = candidate.invariant_authority();
            let control = InvariantPreparationControl::for_transaction(
                input,
                execution.allocation_policy(),
                crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation,
            );
            let boundary = invariants.enforce_commit_boundary_for_selected_branch(
                &selected,
                mutation.state(),
                identity.proposed_version_id(),
                admitted.merged_plan(),
                Some(&identity),
                execution,
                &control,
            )?;
            let mutation_sensitive = invariants.enforce_mutation_sensitive_for_working_state(
                &selected,
                mutation.state(),
                identity.proposed_version_id(),
                admitted.merged_plan(),
                Some(&identity),
                execution,
                &control,
            )?;
            let publication = invariants.enforce_snapshot_publication_for_working_state(
                &selected,
                mutation.state(),
                identity.proposed_version_id(),
                admitted.merged_plan(),
                Some(&identity),
                execution,
                &control,
            )?;
            Ok(AcceptedMergeDraft {
                scope: PreparedAuthorityScope {
                    selected_branch_state: selected.selected().clone(),
                    structural_summary,
                    working_state,
                    mutation_proposal: (identity, mutation),
                    phase_timing,
                },
                boundary,
                mutation_sensitive,
                publication,
            })
        },
    )
}
