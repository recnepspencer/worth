//! Request-funded access to one indexed canonical envelope.
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::runtime::state::subsystems::history::HistorySubsystem;
use std::sync::Arc;

impl HistorySubsystem {
    pub(crate) fn demand_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        match self.commit_artifact(commit_id) {
            Some(artifact) => artifact
                .demand_envelope(execution, 1 << 30)
                .map(Some)
                .map_err(Into::into),
            None => Ok(self.canonical_envelope(commit_id)),
        }
    }
}
