use super::draft_execution::PreparedCommitExecution;
use super::invariant_phase::enforce_commit_boundary_phase;

pub(super) struct BoundaryValidatedCommitExecution {
    prepared: PreparedCommitExecution,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
}

impl BoundaryValidatedCommitExecution {
    pub(super) fn prepared_mut(&mut self) -> &mut PreparedCommitExecution {
        &mut self.prepared
    }

    pub(super) fn push_invariant(
        &mut self,
        invariant: crate::validation::engine::InvariantExecutionResult,
    ) {
        self.invariant_executions.push(invariant);
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        PreparedCommitExecution,
        Vec<crate::validation::engine::InvariantExecutionResult>,
    ) {
        (self.prepared, self.invariant_executions)
    }
}

pub(super) fn validate_commit_boundary(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut prepared: PreparedCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<BoundaryValidatedCommitExecution, crate::transactions::data::TransactionCommitError> {
    let (admitted, proposed_working_state, proposed_version_id, proposal_identity) =
        prepared.boundary_parts();
    let prevalidated = admitted.take_prevalidated_boundary();
    let selected_branch_state = admitted.selected_branch_state().clone();
    let (_, _options, merged_plan, _, commit_log, phase_timing) =
        admitted.phase_view().into_parts();
    let invariant = enforce_commit_boundary_phase(
        runtime,
        commit_log,
        phase_timing,
        &selected_branch_state,
        proposed_working_state,
        proposed_version_id,
        merged_plan,
        Some(proposal_identity),
        prevalidated,
        lease,
        control,
    )?;
    Ok(BoundaryValidatedCommitExecution {
        prepared,
        invariant_executions: vec![invariant],
    })
}
