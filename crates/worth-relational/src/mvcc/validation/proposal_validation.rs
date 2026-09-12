use crate::authority::commit::phases::prepare::{
    prepare_lowered_working_state_scope, prepare_working_state_scope, PreparedWorkingStateScope,
};
use crate::authority::commit::phases::proposed_invariant_state::prepare_proposed_invariant_state;
use crate::runtime::{RelationalPreparationRuntime, RelationalRuntime};
use crate::transactions::data::{
    CommitConflict, CommitValidation, ConflictClass, TransactionCommitError,
};

use super::proposal_invariants::{stale_validated_proposal, validate_proposed_state};
use super::validated_proposal::{
    RelationalMutationInvariantEvidence, StrategyProposalDecoration, ValidatedRelationalProposal,
};

impl RelationalPreparationRuntime {
    pub fn validate_branch_transaction(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.validate_branch_transaction_source(transaction, None, None)
    }

    pub(crate) fn validate_lowered_strategy_proposal(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        selected_branch_state: crate::branch::SelectedRelationalBranchState,
        merged_plan: crate::transactions::data::MergedCommitPlan,
        strategy: StrategyProposalDecoration,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.validate_branch_transaction_source(
            transaction,
            Some((selected_branch_state, merged_plan)),
            Some(strategy),
        )
    }

    fn validate_branch_transaction_source(
        &self,
        mut transaction: crate::mvcc::BranchBoundRelationalTransaction,
        lowered: Option<(
            crate::branch::SelectedRelationalBranchState,
            crate::transactions::data::MergedCommitPlan,
        )>,
        strategy: Option<StrategyProposalDecoration>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        require_not_interrupted(
            &transaction.control,
            &transaction.basis.inner.retention_binding,
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
        )?;
        let complexity_before = self.performance_access().complexity_counters_snapshot();
        let batch_count = transaction.batches().len();
        let basis = transaction.basis.clone();
        let branch = basis.identity().branch_id().clone();
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: basis.identity().runtime_instance_id(),
                },
            )));
        }
        self.history.record_transaction_validation_attempt(&branch);
        if !basis.is_current() {
            return Err(stale_validated_proposal(
                "transaction basis is no longer current",
            ));
        }
        let validated_against_version = basis.observation().version_id();
        let validated_against_commit = basis.observation().commit_id();
        let prepared: PreparedWorkingStateScope = match lowered {
            Some((selected_branch_state, merged_plan)) => prepare_lowered_working_state_scope(
                self,
                &transaction,
                selected_branch_state,
                merged_plan,
            ),
            None => prepare_working_state_scope(self, &mut transaction)?,
        };
        require_not_interrupted(
            &transaction.control,
            &transaction.basis.inner.retention_binding,
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
        )?;
        let transaction_id = transaction.transaction_id;
        let validation_input =
            super::RelationalTransactionValidationInput::from_transaction(&transaction);
        let proposal_identity =
            self.issue_mutation_proposal_identity(transaction_id, &validation_input)?;
        let proposed_version = proposal_identity.proposed_version_id();
        let proposed_working_state = prepare_proposed_invariant_state(
            self,
            &prepared.selected_branch_state,
            &prepared.working_state,
            &prepared.merged_plan,
            prepared.schema_authority.as_ref(),
            proposed_version,
        )?;
        let commit_boundary = self
            .invariant_authority()
            .enforce_commit_boundary_for_selected_branch(
                &prepared.selected_branch_state,
                &proposed_working_state,
                proposed_version,
                &prepared.merged_plan,
                Some(&proposal_identity),
            )?;
        let (mutation_sensitive, publication) = validate_proposed_state(
            self,
            &prepared,
            &proposed_working_state,
            proposed_version,
            Some(&proposal_identity),
        )?;
        let summary = CommitValidation::summarize(&[
            commit_boundary.clone(),
            mutation_sensitive.clone(),
            publication.clone(),
        ]);
        let custom_invariant_execution_receipts =
            super::custom_invariant_execution_receipt::collect_custom_invariant_execution_receipts(
                [&commit_boundary, &mutation_sensitive, &publication],
                &self.schema_contract_runtime.custom_invariant_registries,
            );
        let strategy_commit_artifacts = strategy.as_ref().map(|strategy| {
            let validation_cost =
                crate::commit_strategies::data::StrategyPreviewValidationCostSummary::new(
                    proposed_version,
                    prepared.merged_plan.merged_intents.len(),
                    prepared.structural_summary.touched_partitions.len(),
                    prepared.structural_summary.bulk_entity_slots_reserved,
                    prepared.structural_summary.bulk_relation_slots_reserved,
                    2,
                );
            strategy.artifacts.clone().with_preview_validation(
                summary,
                validation_cost,
                validated_against_commit,
                validated_against_version,
            )
        });
        let strategy_bulk_mutation_batch =
            strategy.and_then(|strategy| strategy.bulk_mutation_batch);
        let complexity_after = self.performance_access().complexity_counters_snapshot();
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
        Ok(ValidatedRelationalProposal {
            mutation_authority: transaction.mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_working_state,
            commit_boundary,
            mutation_sensitive,
            snapshot_publication: publication,
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
            custom_invariant_generation: self.schema_contract_runtime.custom_invariant_generation,
            batch_count,
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
        })
    }
}

impl RelationalRuntime {
    pub fn validate_branch_transaction(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.preparation_runtime_snapshot()
            .validate_branch_transaction(transaction)
    }

    pub(crate) fn validate_lowered_strategy_proposal(
        &self,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        selected_branch_state: crate::branch::SelectedRelationalBranchState,
        merged_plan: crate::transactions::data::MergedCommitPlan,
        strategy: StrategyProposalDecoration,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.preparation_runtime_snapshot()
            .validate_lowered_strategy_proposal(
                transaction,
                selected_branch_state,
                merged_plan,
                strategy,
            )
    }
}

fn require_not_interrupted(
    control: &crate::mvcc::RelationalOperationControl,
    retention_binding: &crate::history::retention::RelationalBranchRetentionBinding,
    boundary: crate::mvcc::RelationalInterruptionBoundary,
) -> Result<(), TransactionCommitError> {
    match control.observe(boundary) {
        Some(interruption) => {
            retention_binding.record_interruption(interruption);
            Err(TransactionCommitError::interrupted(interruption))
        }
        None => Ok(()),
    }
}
