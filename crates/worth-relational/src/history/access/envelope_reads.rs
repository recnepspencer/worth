//! Execution-taking body reads, bounded to one indexed envelope per demand.
use super::HistoryAccess;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, CommittedVersionSummary};
use crate::identity::data::VersionId;
use std::sync::Arc;

impl HistoryAccess<'_> {
    pub(crate) fn demand_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        match self.runtime.history.commit_artifact(commit_id) {
            Some(artifact) => artifact
                .demand_envelope(execution, 1 << 30)
                .map(|envelope| Some(envelope.clone()))
                .map_err(Into::into),
            None => Ok(self.runtime.history.canonical_envelope(commit_id)),
        }
    }

    /// Read one commit's result cardinality under a bounded envelope demand.
    /// Identity-only callers use `immutable_commit_receipt_for_version`.
    pub fn historical_committed_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: VersionId,
    ) -> Result<Option<CommittedVersionSummary>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let Some(receipt) = self.immutable_commit_receipt_for_version(version_id) else {
            return Ok(None);
        };
        Ok(self
            .demand_commit_envelope(execution, receipt.commit_id)?
            .map(|envelope| {
                CommittedVersionSummary::new(
                    envelope.commit.clone(),
                    envelope.patch.authoritative_record_patches.len(),
                )
            }))
    }
}
