//! Seal the validated successor only after candidate preparation has retired.
use super::proposal_source::RelationalProposalSource;
use super::proposal_validation::require_not_interrupted;
use super::validated_proposal::{
    RelationalMutationInvariantEvidence, ValidatedProposalState, ValidatedRelationalProposal,
};
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{CommitValidation, TransactionCommitError};
#[inline(never)]
pub(super) fn complete(
    owner: &RelationalPreparationRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    transaction: crate::mvcc::BranchBoundRelationalTransaction,
    source: RelationalProposalSource,
    round: crate::authority::commit::pipeline::CommitPhaseBacking<
        super::proposal_round::PreparedProposalRound,
    >,
    complexity_before: crate::performance::data::RuntimeComplexityCounters,
    batch_count: usize,
    basis: crate::branch::AdmittedRelationalBranchBasis,
    validated_against_commit: Option<crate::history::data::CommitId>,
    validated_against_version: crate::identity::data::VersionId,
) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
    let super::proposal_round::PreparedProposalRound {
        prepared,
        mutation: proposed_mutation,
        identity: proposal_identity,
        validation_input,
        invariants,
    } = round.into_inner();
    let branch = basis.identity().branch_id().clone();
    require_not_interrupted(
        &transaction.control,
        &transaction.basis.inner.retention_binding,
        crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
    )?;
    let transaction_id = transaction.transaction_id;
    let proposed_version = proposal_identity.proposed_version_id();
    let summary = CommitValidation::summarize(std::slice::from_ref(&invariants));
    let custom_invariant_execution_receipts =
        super::custom_invariant_execution_receipt::collect_custom_invariant_execution_receipts(
            [&invariants],
            &owner.schema_contract_runtime.custom_invariant_registries,
        );
    let strategy = source.into_strategy();
    let strategy_commit_artifacts = strategy.as_ref().map(|strategy| {
        let validation_cost =
            crate::commit_strategies::data::StrategyPreviewValidationCostSummary::new(
                proposed_version,
                prepared.plan.view().commands().merged_intents.len(),
                prepared.structural_summary.touched_partitions.len(),
                prepared.structural_summary.bulk_entity_slots_reserved,
                prepared.structural_summary.bulk_relation_slots_reserved,
                1,
            );
        strategy.artifacts.clone().with_preview_validation(
            summary,
            validation_cost,
            validated_against_commit,
            validated_against_version,
        )
    });
    let strategy_bulk_mutation_batch = strategy.and_then(|strategy| strategy.bulk_mutation_batch);
    let complexity_after = owner.performance_access().complexity_counters_snapshot();
    let validation_complexity_delta =
        crate::performance::operation_complexity_accounting::complexity_delta_without_commit_gauges(
            complexity_before,
            complexity_after,
        );
    require_not_interrupted(
        &transaction.control,
        &transaction.basis.inner.retention_binding,
        crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
    )?;
    // All inner and outer abandoning probes have completed. Transfer the exact
    // draft and its existing coupled posture to validated proposal custody.
    let proposed_mutation = proposed_mutation.finish();
    let coupled_publication = transaction.authored_coupling.as_ref().and_then(|binding| {
        binding
            .completed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    });
    ValidatedRelationalProposal::retain(
        execution,
        ValidatedProposalState {
            caller_association: None,
            required_output_publication: coupled_publication,
            required_output_materialization: None,
            commit_work_budget: owner.commit_work_budget.clone(),
            mutation_authority: transaction.mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_mutation,
            invariants,
            evidence: RelationalMutationInvariantEvidence {
                branch,
                proposed_version,
                proposal_identity: proposal_identity.clone(),
                summary,
                custom_invariant_execution_receipts,
            },
            proposal_identity,
            validated_against_commit,
            validated_against_version,
            validated_against_branch_version: basis.descriptor().truth_version(),
            receiver_epoch: owner.configuration_epoch.clone(),
            execution_contracts: owner.execution_contracts.clone(),
            custom_invariant_generation: owner.schema_contract_runtime.custom_invariant_generation,
            batch_count,
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
        },
    )
}
