use crate::history::data::{BranchId, CommitId};
use crate::inspection::data::{
    CommitInspection, InspectionAccessPath, InspectionOrigin, RecentCommitInspectionRequest,
    RecentCommitInspectionWindow,
};
use crate::publication::patch::data::ordered_aspect_keys;

use super::access::InspectionAccess;

impl<'runtime> InspectionAccess<'runtime> {
    pub fn inspect_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit_id: CommitId,
    ) -> Result<Option<CommitInspection>, crate::execution::RelationalExecutionDenialCause> {
        self.count_commit_read();
        let Some(envelope) = self
            .runtime
            .history()
            .demand_commit_envelope(execution, commit_id)?
        else {
            return Ok(None);
        };
        Ok(Some(CommitInspection {
            commit: envelope.commit.clone(),
            changed_records: envelope
                .patch
                .authoritative_record_patches
                .iter()
                .map(|record| record.target.clone())
                .collect(),
            lineage_event_ids: envelope
                .lineage_digest_basis()
                .canonical_event_ids()
                .to_vec(),
            lineage_events: envelope.lineage_events().to_vec(),
            lineage_digest_basis: envelope.lineage_digest_basis().clone(),
            lineage_artifact_counters: envelope.lineage_artifact_counters(),
            derived_index_artifacts: envelope.derived_index_artifacts().clone(),
            changed_aspects: ordered_aspect_keys(
                envelope
                    .patch
                    .authoritative_record_patches
                    .iter()
                    .flat_map(|record| record.authoritative_changed_aspect_keys().cloned()),
            ),
            origin: InspectionOrigin::CanonicalCommitStorage,
            access_path: InspectionAccessPath::CommitIndexRead,
        }))
    }

    pub fn inspect_recent_commits(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &RecentCommitInspectionRequest,
    ) -> Result<RecentCommitInspectionWindow, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        let limit = usize::try_from(request.limit).map_err(|_| {
            crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded
        })?;
        let mut commits = Vec::new();
        for commit_id in self.recent_commit_ids(request.branch_id.as_ref(), limit) {
            if let Some(commit) = self.inspect_commit(execution, commit_id)? {
                commits.push(commit);
            }
        }
        let branch_head = request
            .branch_id
            .as_ref()
            .and_then(|branch_id| self.branch_head_ref(branch_id));
        Ok(RecentCommitInspectionWindow {
            branch_head,
            commits,
            origin: InspectionOrigin::CanonicalCommitStorage,
            access_path: InspectionAccessPath::CommitIndexRead,
        })
    }

    pub fn inspect_branch_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: &BranchId,
    ) -> Result<Option<CommitInspection>, crate::execution::RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let Some(head) = self.branch_head_ref(branch_id) else {
            return Ok(None);
        };
        self.inspect_commit(execution, head.commit_id)
    }
}
