//! Branch-bound invariant planning owned by a live transaction.

use crate::branch::SelectedRelationalBranchState;
use crate::transactions::data::{CommitConflict, TransactionCommitError};
use crate::validation::engine::InvariantExecutionResult;

impl crate::mvcc::BranchBoundRelationalTransaction {
    /// Evaluate commit-boundary planning against this transaction's exact
    /// owner-issued branch basis.
    pub fn commit_boundary_plan(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )?;
        let (selected_state, merged_plan) = self
            .branch_bound_invariant_plan(execution, runtime)
            .map_err(TransactionCommitError::conflict)?;
        runtime
            .validation()
            .commit_boundary_for_selected_branch_plan(execution, &selected_state, &merged_plan)
    }

    /// Evaluate graph-composition planning against this transaction's exact
    /// owner-issued branch basis.
    pub fn graph_composition_plan(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<InvariantExecutionResult, TransactionCommitError> {
        crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Validation,
        )?;
        let (selected_state, merged_plan) = self
            .branch_bound_invariant_plan(execution, runtime)
            .map_err(TransactionCommitError::conflict)?;
        runtime
            .validation()
            .graph_composition_for_selected_branch_plan(execution, &selected_state, &merged_plan)
    }

    fn branch_bound_invariant_plan(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<
        (
            SelectedRelationalBranchState,
            crate::transactions::data::MergedCommitPlan,
        ),
        CommitConflict,
    > {
        self.ensure_current_basis_for_runtime(runtime)?;
        let selected_state = SelectedRelationalBranchState::from_admitted_basis(&self.basis);
        let merged_plan = self
            .merged_plan(runtime, execution.allocation_policy())?
            .clone();
        Ok((selected_state, merged_plan))
    }
}
