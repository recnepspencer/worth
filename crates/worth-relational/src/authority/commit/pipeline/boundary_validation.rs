use super::draft_execution::PreparedCommitExecution;
use super::invariant_phase::enforce_commit_boundary_phase;
use super::phase_backing::CommitPhaseBacking;

pub(super) struct BoundaryValidatedCommitExecution {
    prepared: PreparedCommitExecution,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
}

impl BoundaryValidatedCommitExecution {
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    prepared: CommitPhaseBacking<PreparedCommitExecution>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<
    CommitPhaseBacking<BoundaryValidatedCommitExecution>,
    crate::transactions::data::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let result = {
        let mut prepared = prepared.into_inner();
        super::consult_stage(execution, super::CommitStage::Boundary)?;
        let (
            admitted,
            proposed_working_state,
            proposed_version_id,
            proposal_identity,
            aggregate_source,
        ) = prepared.boundary_parts();
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
            aggregate_source,
            prevalidated,
            execution,
            control,
        )?;
        BoundaryValidatedCommitExecution {
            prepared,
            invariant_executions: vec![invariant],
        }
    };
    CommitPhaseBacking::new(execution, result)
}
