//! Execution-taking body reads, bounded to one indexed envelope per demand.
use super::HistoryAccess;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, CommittedVersionSummary};
use crate::identity::data::VersionId;
use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::committed_version::CommittedVersionSummaryOwnerPort as _;

impl HistoryAccess<'_> {
    pub(crate) fn demand_commit_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, RelationalExecutionDenialCause> {
        self.runtime
            .history
            .demand_commit_envelope(execution, commit_id)
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
