use super::boundary_validation::BoundaryValidatedCommitExecution;
use super::draft_execution::PreparedCommitState;
use super::phase_backing::CommitPhaseBacking;
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
    // Retain the proposal and its selected/source backing through history binding.
    pub(super) _aggregate: Option<crate::aggregate_identity::AggregateProposal>,
    pub(super) source_interpretation:
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
}

impl MutatedCommitExecution {
    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.prepared.version_id()
    }
}

pub(super) fn mutate_commit_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    validated: CommitPhaseBacking<BoundaryValidatedCommitExecution>,
    _control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<
    CommitPhaseBacking<MutatedCommitExecution>,
    crate::transactions::data::TransactionCommitError,
> {
    let result = {
        let validated = validated.into_inner();
        use crate::transactions::data::CommitPhase;
        super::consult_stage(execution, super::CommitStage::Mutation)?;
        let (prepared, invariant_executions) = validated.into_parts();
        let (
            mut prepared,
            AppliedMutationDraft {
                state: working_state,
                outcome,
                identities: mutation_identities,
                aggregate,
                source_interpretation,
            },
        ) = prepared.into_parts();
        telemetry::record_preparation_telemetry(runtime, outcome.preparation_telemetry);
        crate::authority::commit::phases::prepare::record_mutation_counters(
            runtime,
            &working_state,
        );
        let (_, _, _, _, commit_log, phase_timing) = prepared.admitted.phase_view().into_parts();
        commit_log.begin_phase(CommitPhase::AuthoritativeMutation);
        let started = std::time::Instant::now();
        commit_log.complete_phase(CommitPhase::AuthoritativeMutation);
        phase_timing.authoritative_mutation_micros = super::rejection::elapsed_micros(started);
        MutatedCommitExecution {
            prepared,
            working_state,
            invariant_executions,
            outcome,
            mutation_identities,
            _aggregate: aggregate,
            source_interpretation,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
#[cfg(test)]
#[path = "mutation_execution_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
