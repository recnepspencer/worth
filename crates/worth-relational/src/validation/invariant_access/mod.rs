mod execution;
mod metadata;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

use crate::branch::SelectedRelationalBranchState;
use crate::runtime::RelationalRuntime;
use crate::transactions::data::MergedCommitPlan;
#[cfg(test)]
use crate::validation::engine::HarnessAuditMode;
use crate::validation::engine::{InvariantExecutionResult, InvariantRequestProfile};

impl RelationalRuntime {
    pub(crate) fn invariant_access(&self) -> InvariantAccess<'_> {
        InvariantAccess::new(self)
    }
}

pub struct InvariantAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
    view: crate::validation::engine::InvariantRuntimeView<'runtime>,
}

impl<'runtime> InvariantAccess<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self {
            runtime,
            view: crate::validation::engine::InvariantRuntimeView::from_runtime(runtime),
        }
    }

    #[cfg(test)]
    pub fn harness_audit(&self, mode: HarnessAuditMode) -> InvariantExecutionResult {
        let backing = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&backing);
        mode.request_profile().map_or_else(
            || {
                InvariantExecutionResult::skipped(self.execution_metadata(
                    InvariantRequestProfile::HarnessAudit,
                    crate::validation::engine::InvariantObservationKind::Committed,
                    self.runtime.current_version_id(),
                    self.runtime.current_version_id(),
                    None,
                    None,
                    crate::validation::data::InvariantGroupSet::empty(),
                    crate::validation::data::InvariantCostClass::Global,
                    crate::validation::engine::InvariantExecutionDisposition::SkippedByMayBreakMask,
                    None,
                ))
            },
            |profile| {
                self.execute_for_runtime(execution, profile)
                    .expect("audit fixture request")
            },
        )
    }

    pub fn mutation_sensitive_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        self.execute_for_runtime(execution, InvariantRequestProfile::MutationSensitive)
    }

    pub fn snapshot_publication_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        self.execute_for_runtime(execution, InvariantRequestProfile::SnapshotPublication)
    }

    pub fn certification_state(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        self.execute_for_runtime(execution, InvariantRequestProfile::CertificationBoundary)
    }

    pub(crate) fn commit_boundary_for_selected_branch_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected_state: &SelectedRelationalBranchState,
        merged_plan: &MergedCommitPlan,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        self.execute_for_selected_branch_committed_plan(
            execution,
            InvariantRequestProfile::CommitBoundary,
            selected_state,
            merged_plan,
        )
    }

    pub(crate) fn graph_composition_for_selected_branch_plan(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected_state: &SelectedRelationalBranchState,
        merged_plan: &MergedCommitPlan,
    ) -> Result<InvariantExecutionResult, crate::transactions::data::TransactionCommitError> {
        self.execute_for_selected_branch_committed_plan(
            execution,
            InvariantRequestProfile::GraphComposition,
            selected_state,
            merged_plan,
        )
    }
}
