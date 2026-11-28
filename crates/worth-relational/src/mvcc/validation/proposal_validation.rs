use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::runtime::{RelationalPreparationRuntime, RelationalRuntime};
use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
use crate::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

use super::proposal_invariants::stale_validated_proposal;
use super::proposal_source::RelationalProposalSource;
use super::validated_proposal::ValidatedRelationalProposal;

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
                RelationalProposalSource::Transaction,
                context,
            )
            .map_err(crate::execution::allocation_commit_error)
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

    fn validate_branch_transaction_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        source: RelationalProposalSource,
        context: Option<&dyn crate::validation::candidate_source::ManagedIndexSourceContextFactory>,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        self.select_home_contracts(
            &transaction.basis,
            transaction.home_target.as_ref(),
            transaction.recorded_result_schema(),
        )?
        .validate_selected_branch_transaction_source(
            execution,
            transaction,
            source,
            context,
        )
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
                    crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::HomeProgramTransition(cause),
                )
            })?;
        crate::execution::transaction_stage(execution, self.commit_work_budget.as_ref(), || {
            transaction.require_home_program_body(execution)
        })?;
        require_not_interrupted(
            &transaction.control,
            &transaction.basis.owner_inner().retention_binding,
            crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::ProposalValidation,
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
        crate::execution::transaction_stage(execution, self.commit_work_budget.as_ref(), || {
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
            transaction.prepare_home_target_coverage(execution, self)
        })?;
        let validated_against_version = basis.observation().version_id();
        let validated_against_commit = basis.observation().commit_id();
        let round = self.prepare_proposal_round(execution, &mut transaction, &source, context)?;
        super::proposal_completion::complete(
            self,
            execution,
            transaction,
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
}

pub(in crate::mvcc) fn require_not_interrupted(
    control: &crate::runtime::operation_control::RelationalOperationControl,
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

#[cfg(test)]
#[path = "proposal_validation/native_report_tests.rs"]
mod native_report_tests;
