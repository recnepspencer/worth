use crate::runtime::{RelationalPreparationRuntime, RelationalRuntime};
use crate::transactions::data::{CommitConflict, ConflictClass, TransactionCommitError};

use super::proposal_invariants::stale_validated_proposal;
use super::proposal_source::RelationalProposalSource;
use super::validated_proposal::{StrategyProposalDecoration, ValidatedRelationalProposal};

impl RelationalPreparationRuntime {
    pub(crate) fn validate_consulted_branch_transaction(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.validate_consulted_branch_transaction_with_source_context(execution, transaction, None)
    }

    pub(crate) fn validate_consulted_branch_transaction_with_source_context(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let runtime = if execution.is_leased() {
            self.with_commit_work_budget(None)
        } else {
            self.clone()
        };
        runtime
            .validate_branch_transaction_source(
                execution,
                transaction,
                RelationalProposalSource::Transaction {
                    lowered: None,
                    strategy: None,
                },
                context,
            )
            .map_err(crate::execution::allocation_commit_error)
    }

    pub(crate) fn validate_consulted_lowered_strategy_proposal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        selected_branch_state: crate::branch::SelectedRelationalBranchState,
        merged_plan: crate::transactions::data::MergedCommitPlan,
        strategy: StrategyProposalDecoration,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let runtime = if execution.is_leased() {
            self.with_commit_work_budget(None)
        } else {
            self.clone()
        };
        runtime.validate_branch_transaction_source(
            execution,
            transaction,
            RelationalProposalSource::Transaction {
                lowered: Some((selected_branch_state, merged_plan)),
                strategy: Some(strategy),
            },
            None,
        )
    }

    pub(in crate::mvcc) fn validate_consulted_retained_entity_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        retained: crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let runtime = if execution.is_leased() {
            self.with_commit_work_budget(None)
        } else {
            self.clone()
        };
        runtime.validate_branch_transaction_source(
            execution,
            transaction,
            RelationalProposalSource::RetainedEntitySelection(retained),
            None,
        )
    }

    pub(in crate::mvcc) fn validate_consulted_owned_graph_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        retained: crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.validate_branch_transaction_source(
            execution,
            transaction,
            RelationalProposalSource::OwnedGraphSelection(retained),
            None,
        )
    }

    fn validate_branch_transaction_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        source: RelationalProposalSource,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.select_home_contracts(&transaction.basis, transaction.home_target.as_ref())?
            .validate_selected_branch_transaction_source(execution, transaction, source, context)
    }

    fn validate_selected_branch_transaction_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut transaction: crate::mvcc::BranchBoundRelationalTransaction,
        source: RelationalProposalSource,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.durability_admission
            .admit_home_program_write(
                transaction.home_step.as_deref(),
                transaction.recovered_codec_publication_custody.as_deref(),
            )
            .map_err(|cause| {
                TransactionCommitError::publication_denied(
                    crate::mvcc::RelationalPublicationDenial::HomeProgramTransition(cause),
                )
            })?;
        transaction.require_home_program_body(execution)?;
        require_not_interrupted(
            &transaction.control,
            &transaction.basis.inner.retention_binding,
            crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
        )?;
        let complexity_before = self.performance_access().complexity_counters_snapshot();
        let batch_count = transaction.batches().len();
        let basis = transaction.basis.clone();
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(TransactionCommitError::conflict(CommitConflict::new(
                ConflictClass::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: basis.identity().runtime_instance_id(),
                },
            )));
        }
        self.history
            .record_transaction_validation_attempt(basis.identity().branch_id());
        if !basis.is_current() {
            return Err(stale_validated_proposal(
                "transaction basis is no longer current",
            ));
        }
        self.durability_admission
            .require_open()
            .map_err(TransactionCommitError::durability_unresolved)?;
        if transaction.schema_authority_input.is_some() {
            // Schema reframing is a declared whole-root operation; its capture
            // must not reconstruct a cold region after candidate acceptance.
            transaction.demand_preparation_regions(
                execution,
                &self.services.symbols,
                &self.index_definitions,
                &self.mirror_residency,
            )?;
        } else {
            transaction.demand_preparation_inputs(
                execution,
                &self.services.symbols,
                &self.index_definitions,
            )?;
        }
        let validated_against_version = basis.observation().version_id();
        transaction.prepare_home_target_coverage(execution, self)?;
        let validated_against_commit = basis.observation().commit_id();
        let round = self.prepare_proposal_round(execution, &mut transaction, &source, context)?;
        super::proposal_completion::complete(
            self,
            execution,
            transaction,
            source,
            round,
            complexity_before,
            batch_count,
            basis,
            validated_against_commit,
            validated_against_version,
        )
    }
}

impl RelationalRuntime {
    pub fn validate_branch_transaction(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        worth_execution::run_with_execution_stack(move || {
            self.validate_branch_transaction_on_execution_stack(execution, transaction)
        })
    }

    #[inline(never)]
    fn validate_branch_transaction_on_execution_stack(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )?;
        self.preparation_runtime_snapshot()
            .validate_consulted_branch_transaction(execution, transaction)
    }

    /// Validate with metadata independently bound to every actual proposal round.
    /// The factory is borrowed; the caller retains its outer fence ownership.
    pub fn validate_branch_transaction_with_source_context(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        context: &dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        worth_execution::run_with_execution_stack(move || {
            self.validate_branch_transaction_with_source_context_on_execution_stack(
                execution,
                transaction,
                context,
            )
        })
    }

    #[inline(never)]
    fn validate_branch_transaction_with_source_context_on_execution_stack(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        context: &dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )?;
        self.preparation_runtime_snapshot()
            .validate_consulted_branch_transaction_with_source_context(
                execution,
                transaction,
                Some(context),
            )
    }

    pub(crate) fn validate_consulted_lowered_strategy_proposal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        selected_branch_state: crate::branch::SelectedRelationalBranchState,
        merged_plan: crate::transactions::data::MergedCommitPlan,
        strategy: StrategyProposalDecoration,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.preparation_runtime_snapshot()
            .validate_consulted_lowered_strategy_proposal(
                execution,
                transaction,
                selected_branch_state,
                merged_plan,
                strategy,
            )
    }
}

pub(in crate::mvcc) fn require_not_interrupted(
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
