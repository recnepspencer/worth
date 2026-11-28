use crate::history::data::CommitId;
use crate::history::data::SharedCanonicalCommitEnvelope;
use crate::runtime::RelationalRuntime;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;

impl RelationalRuntime {
    pub fn replay_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<
        Option<SharedCanonicalCommitEnvelope>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        self.history()
            .demand_commit_envelope(execution, commit_id)
            .map(|envelope| envelope.map(SharedCanonicalCommitEnvelope::from_arc))
    }
}
