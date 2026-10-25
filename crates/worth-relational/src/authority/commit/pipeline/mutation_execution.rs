use super::boundary_validation::BoundaryValidatedCommitExecution;
use super::draft_execution::PreparedCommitState;
use crate::authority::commit::phases::applied_mutation::{
    AppliedMutationDraft, PendingMutationIdentities,
};
use crate::authority::mutation::MutationApplyOutcome;
use crate::storage::overlay::WorkingState;

mod telemetry;

pub(super) struct MutatedCommitExecution {
    pub(super) prepared: PreparedCommitState,
    pub(super) working_state: WorkingState,
    pub(super) invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    pub(super) outcome: MutationApplyOutcome,
    pub(super) mutation_identities: PendingMutationIdentities,
}

impl MutatedCommitExecution {
    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.prepared.version_id()
    }
}

pub(super) fn mutate_commit_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    validated: BoundaryValidatedCommitExecution,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<MutatedCommitExecution, crate::transactions::data::TransactionCommitError> {
    use crate::transactions::data::CommitPhase;
    super::consult_stage(execution, super::CommitStage::Mutation)?;
    let (prepared, mut invariant_executions) = validated.into_parts();
    let (
        mut prepared,
        AppliedMutationDraft {
            state: working_state,
            outcome,
            identities: mutation_identities,
        },
    ) = prepared.into_parts();
    let selected = prepared.selected_branch_state().clone();
    let version = prepared.version_id();
    let prevalidated = prepared.admitted.take_prevalidated_mutation_sensitive();
    telemetry::record_preparation_telemetry(runtime, outcome.preparation_telemetry);
    crate::authority::commit::phases::prepare::record_mutation_counters(runtime, &working_state);
    let (_, _, merged_plan, _, commit_log, phase_timing) =
        prepared.admitted.phase_view().into_parts();
    commit_log.begin_phase(CommitPhase::AuthoritativeMutation);
    let started = std::time::Instant::now();
    let invariant = match prevalidated {
        Some(invariant) => invariant,
        None => runtime
            .invariant_authority()
            .enforce_mutation_sensitive_for_working_state(
                &selected,
                &working_state,
                version,
                merged_plan,
                Some(&prepared.proposal_identity),
                execution,
                control,
            )
            .map_err(|error| {
                super::rejection::attach_rejection(
                    commit_log,
                    CommitPhase::AuthoritativeMutation,
                    error,
                )
            })?,
    };
    commit_log.record_invariant_outcomes(&invariant);
    commit_log.complete_phase(CommitPhase::AuthoritativeMutation);
    phase_timing.authoritative_mutation_micros = super::rejection::elapsed_micros(started);
    invariant_executions.push(invariant);
    Ok(MutatedCommitExecution {
        prepared,
        working_state,
        invariant_executions,
        outcome,
        mutation_identities,
    })
}
