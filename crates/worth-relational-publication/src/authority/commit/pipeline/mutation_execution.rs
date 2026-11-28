use worth_relational_contracts::internal_port::execution::monotonic_time::monotonic_now;
use super::boundary_validation::BoundaryValidatedCommitExecution;
use super::draft_execution::PreparedCommitState;
use super::phase_backing::CommitPhaseBacking;
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome;
use worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub mod telemetry;

pub(super) struct MutatedCommitExecution {
    pub(super) prepared: PreparedCommitState,
    pub(super) working_state: WorkingState,
    pub(super) content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub(super) invariant_executions: Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
    pub(super) outcome: MutationApplyOutcome,
    pub(super) mutation_identities: PendingMutationIdentities,
    // Retain the proposal and its selected/source backing through history binding.
    pub(super) _aggregate: Option<worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposal>,
    pub(super) source_interpretation:
        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
}

impl MutatedCommitExecution {
    pub(super) fn version_id(&self) -> contracts::identity::data::primitives::VersionId {
        self.prepared.version_id()
    }
}

pub(super) fn mutate_commit_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    instrumentation: &worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    validated: CommitPhaseBacking<BoundaryValidatedCommitExecution>,
) -> Result<
    CommitPhaseBacking<MutatedCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    let result = {
        let validated = validated.into_inner();
        use contracts::transactions::data::commit_log::CommitPhase;
        super::consult_stage(execution, super::CommitStage::Mutation)?;
        let (prepared, invariant_executions) = validated.into_parts();
        let (
            mut prepared,
            AppliedMutationDraft {
                state: working_state,
                content,
                outcome,
                identities: mutation_identities,
                aggregate,
                source_interpretation,
            },
        ) = prepared.into_parts();
        telemetry::record_preparation_telemetry(instrumentation, outcome.preparation_telemetry);
        crate::authority::commit::phases::prepare::working_state_metrics::record_mutation_counters(
            instrumentation,
            &working_state,
        );
        let (_, _, _, commit_log, phase_timing) = prepared.admitted.phase_view().into_parts();
        commit_log.begin_phase(CommitPhase::AuthoritativeMutation);
        let started = monotonic_now();
        commit_log.complete_phase(CommitPhase::AuthoritativeMutation);
        phase_timing.authoritative_mutation_micros = super::rejection::elapsed_micros(started);
        MutatedCommitExecution {
            prepared,
            working_state,
            content,
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
pub mod inline_type_budget;
