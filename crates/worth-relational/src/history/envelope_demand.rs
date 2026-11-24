//! Request-funded canonical history reads from the selected branch ledger and performed routes.
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::RelationalCommitReceipt;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub(crate) trait PerformedCanonicalHistory {
    fn resident_envelope(&self, commit: CommitId) -> Option<Arc<CanonicalCommitEnvelope>>;
    fn by_version(&self, version: VersionId) -> Option<Arc<CanonicalCommitEnvelope>>;
    fn by_patch(&self, position: PatchStreamPosition) -> Option<Arc<CanonicalCommitEnvelope>>;
    fn latest_patch(&self) -> Option<PatchStreamPosition>;
    fn positioned(&self, commit: CommitId) -> Option<Arc<PositionedCanonicalCommit>>;
    fn find_version(
        &self,
        version: VersionId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<CommitId>, RelationalExecutionDenialCause>;
}

#[derive(Clone, Copy)]
pub(crate) struct CanonicalHistoryRead<'a> {
    pub(crate) branch: &'a HistorySubsystem,
    pub(crate) performed: &'a dyn PerformedCanonicalHistory,
}

impl CanonicalHistoryRead<'_> {
    pub(crate) fn receipt_for_version(self, version: VersionId) -> Option<RelationalCommitReceipt> {
        self.branch
            .commit_artifact_for_version(version)
            .map(|artifact| artifact.receipt())
            .or_else(|| {
                self.performed
                    .by_version(version)
                    .map(|envelope| envelope.commit.clone())
            })
    }

    pub(crate) fn latest_patch_stream_position(self) -> Option<PatchStreamPosition> {
        self.branch
            .latest_recorded_patch_position()
            .max(self.performed.latest_patch())
    }

    pub(crate) fn latest_receipt(self) -> Option<RelationalCommitReceipt> {
        self.latest_patch_stream_position()
            .and_then(|position| {
                self.performed
                    .by_patch(position)
                    .map(|envelope| envelope.commit.clone())
                    .or_else(|| {
                        self.branch
                            .recorded_commit_at_patch_position(position)
                            .and_then(|commit| self.branch.commit_artifact(commit))
                            .map(|artifact| artifact.receipt())
                    })
            })
            .or_else(|| {
                self.branch
                    .latest_commit_artifact()
                    .map(|artifact| artifact.receipt())
            })
    }

    pub(crate) fn canonical_envelope(
        self,
        commit: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.branch
            .commit_artifact(commit)
            .and_then(|artifact| artifact.resident_envelope())
            .or_else(|| self.performed.resident_envelope(commit))
    }

    pub(crate) fn demand_commit_envelope(
        self,
        execution: &ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        match self.branch.commit_artifact(commit) {
            Some(artifact) => artifact
                .demand_envelope(execution, 1 << 30)
                .map(Some)
                .map_err(Into::into),
            None => Ok(self.performed.resident_envelope(commit)),
        }
    }
}
