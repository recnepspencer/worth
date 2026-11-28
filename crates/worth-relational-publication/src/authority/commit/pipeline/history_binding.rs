use super::history_resolution_phase::resolve_authoritative_history_phase;
use super::mutation_execution::MutatedCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use crate::authority::commit::phases::schema_continuity::{
    resolve_schema_continuity,
};
use contracts::transactions::data::commit_log::CommitPhase;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;

pub(super) struct HistoryBoundCommitExecution {
    mutated: MutatedCommitExecution,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    schema_continuity: crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
}

impl HistoryBoundCommitExecution {
    pub(super) fn mutated_mut(&mut self) -> &mut MutatedCommitExecution {
        &mut self.mutated
    }

    pub(super) fn candidate_symbols(
        &self,
    ) -> &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings{
        &self.mutated.mutation_identities.symbols
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        MutatedCommitExecution,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,

    ) {
        (
            self.mutated,
            self.history,
            self.schema_continuity,
        )
    }
}

/// Required request borrow proves this work belongs to the caller, without retaining it.
pub(super) fn bind_commit_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: crate::authority::commit::phases::schema_continuity::inputs::SchemaContinuityInputs<'_>,
    mutated: CommitPhaseBacking<MutatedCommitExecution>,
) -> Result<
    CommitPhaseBacking<HistoryBoundCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let result = {
        let mut mutated = mutated.into_inner();
        super::consult_stage(execution, super::CommitStage::History)?;
        let version_id = mutated.version_id();
        let commit_id = mutated.prepared.proposal_identity.commit_id();
        let admitted = &mut mutated.prepared.admitted;
        let (_, validation_input, _, commit_log, phase_timing) = admitted.phase_view().into_parts();
        let history = resolve_authoritative_history_phase(
            commit_log,
            phase_timing,
            validation_input.basis(),
            version_id,
            commit_id,
        )?;
        let schema_continuity =
            resolve_schema_continuity(execution, inputs, &history.branch_id, validation_input)
                .map_err(|error| {
                    super::rejection::attach_rejection(
                        commit_log,
                        CommitPhase::ArtifactAssembly,
                        error,
                    )
                })?;
        HistoryBoundCommitExecution {
            mutated,
            history,
            schema_continuity,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
