//! Candidate construction returns before invariant and aggregate preparation.
use super::proposal_source::RelationalProposalSource;
use crate::authority::commit::phases::applied_mutation::{
    apply_mutation_draft, apply_retained_entity_selection_draft, AppliedMutationDraft,
};
use crate::authority::commit::phases::prepare::{
    prepare_lowered_working_state_scope, prepare_working_state_scope, PreparedWorkingStateScope,
};
use crate::authority::commit::pipeline::{CommitPhaseBacking, CommitPhaseReservation};
use crate::mvcc::{
    BranchBoundRelationalTransaction, RelationalMutationProposalIdentity,
    RelationalTransactionValidationInput,
};
use crate::runtime::RelationalPreparationRuntime;

use super::proposal_round::PreparedProposalRound;
use crate::authority::mutation::MutationApplyDenial;
pub(super) struct PreparedProposalCandidate {
    prepared: PreparedWorkingStateScope,
    mutation: AppliedMutationDraft,
    identity: RelationalMutationProposalIdentity,
    validation_input: RelationalTransactionValidationInput,
}
#[inline(never)]
pub(super) fn prepare(
    owner: &RelationalPreparationRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    transaction: &mut BranchBoundRelationalTransaction,
    source: &RelationalProposalSource,
    selected: crate::branch::SelectedRelationalBranchRead<'_>,
    context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    mut symbols: crate::runtime::PendingSymbolMeanings,
    round: &RelationalPreparationRuntime,
) -> Result<CommitPhaseBacking<PreparedProposalCandidate>, MutationApplyDenial> {
    let storage = CommitPhaseReservation::prepare(execution)?;
    transaction.ensure_current_basis(owner)?;
    if let Some(prior) = &transaction.pending_symbols {
        symbols.adopt(prior)?;
    }
    transaction.validate_staged_branch_locality(&selected, symbols.interner())?;
    let input = RelationalTransactionValidationInput::from_transaction(transaction);
    let (identity, source_interpretation) =
        owner.issue_mutation_proposal_identity(transaction.transaction_id, &input)?;
    crate::execution::transaction_stage(execution, owner.commit_work_budget.as_ref(), || {
        source_interpretation.bind_context(execution, context)
    })?;
    transaction.reserve_preparation_client_keys(
        &mut symbols,
        owner.config.identity.client_key_symbol_policy,
        execution.allocation_policy(),
    )?;
    let runtime = round.with_candidate_symbols(&symbols);
    let prepared = match source {
        RelationalProposalSource::Transaction {
            lowered: Some((_, plan)),
            ..
        } => prepare_lowered_working_state_scope(
            &runtime,
            transaction,
            selected,
            crate::mvcc::OwnedNormalizedPlan::copy_for_request(plan, execution)?,
        ),
        RelationalProposalSource::Transaction { lowered: None, .. } => {
            prepare_working_state_scope(&runtime, transaction, selected, execution)?
        }
        RelationalProposalSource::RetainedEntitySelection(retained) => {
            crate::authority::commit::phases::prepare::retained_entity_selection::prepare(
                &runtime,
                transaction,
                selected,
                retained,
                execution.allocation_policy(),
            )?
        }
    };
    let validation_input = RelationalTransactionValidationInput::from_transaction(transaction);
    let mutation = match prepared.plan.retained() {
        Some(retained) => apply_retained_entity_selection_draft(
            execution,
            &runtime,
            &selected,
            &prepared.working_state,
            retained,
            prepared.schema_authority.as_ref(),
            &identity,
            symbols,
            source_interpretation,
            &validation_input.content,
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
    Ok(storage.fill(PreparedProposalCandidate {
        prepared,
        mutation,
        identity,
        validation_input,
    }))
}
#[inline(never)]
pub(super) fn validate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    selected: crate::branch::SelectedRelationalBranchRead<'_>,
    round: &RelationalPreparationRuntime,
    candidate: CommitPhaseBacking<PreparedProposalCandidate>,
    storage: CommitPhaseReservation<PreparedProposalRound>,
) -> Result<CommitPhaseBacking<PreparedProposalRound>, MutationApplyDenial> {
    let PreparedProposalCandidate {
        prepared,
        mut mutation,
        identity,
        validation_input,
    } = candidate.into_inner();
    let runtime = round.with_candidate_symbols(mutation.candidate_symbols());
    mutation.prepare_managed_sources(
        *execution,
        &runtime,
        selected,
        &identity,
        (validation_input.merge_parent_bases().is_empty() && prepared.plan.retained().is_none())
            .then_some(prepared.plan.view().commands()),
    )?;
    let control = crate::validation::engine::InvariantPreparationControl::for_transaction(
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
            prepared.plan.view(),
            Some(&identity),
            mutation.aggregate_source(selected),
            execution,
            &control,
        )?;
    let (mutation_sensitive, publication) = super::proposal_invariants::validate_proposed_state(
        &runtime,
        &prepared,
        &selected,
        &mutation,
        identity.proposed_version_id(),
        Some(&identity),
        execution,
        &control,
    )?;
    mutation.prepare_aggregate(execution, round, selected, prepared.plan.view().commands())?;
    Ok(storage.fill(PreparedProposalRound {
        prepared,
        mutation,
        identity,
        validation_input,
        commit_boundary,
        mutation_sensitive,
        publication,
    }))
}
