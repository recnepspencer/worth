//! Branch-bound invariant planning owned by a live transaction.

use crate::branch::SelectedRelationalBranchState;
use crate::transactions::data::CommitConflict;
use crate::validation::engine::InvariantExecutionResult;

impl crate::mvcc::BranchBoundRelationalTransaction {
    /// Evaluate commit-boundary planning against this transaction's exact
    /// owner-issued branch basis.
    pub fn commit_boundary_plan(
        &mut self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<InvariantExecutionResult, CommitConflict> {
        let (selected_state, merged_plan) = self.branch_bound_invariant_plan(runtime)?;
        Ok(runtime
            .validation()
            .commit_boundary_for_selected_branch_plan(&selected_state, &merged_plan))
    }

    /// Evaluate graph-composition planning against this transaction's exact
    /// owner-issued branch basis.
    pub fn graph_composition_plan(
        &mut self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<InvariantExecutionResult, CommitConflict> {
        let (selected_state, merged_plan) = self.branch_bound_invariant_plan(runtime)?;
        Ok(runtime
            .validation()
            .graph_composition_for_selected_branch_plan(&selected_state, &merged_plan))
    }

    fn branch_bound_invariant_plan(
        &mut self,
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
            .merged_plan(
                runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )?
            .clone();
        Ok((selected_state, merged_plan))
    }
}
