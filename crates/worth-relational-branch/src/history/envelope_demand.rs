//! Request-funded canonical history reads from the selected branch ledger and performed routes.
use crate::execution::RelationalExecutionDenialCause;
use crate::history::data::RelationalCommitReceipt;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, MapKernelContext};

pub trait PerformedCanonicalHistory {
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
pub struct CanonicalHistoryRead<'a> {
    pub branch: &'a HistorySubsystem,
    pub performed: &'a dyn PerformedCanonicalHistory,
}

impl CanonicalHistoryRead<'_> {
    pub fn receipt_for_version(self, version: VersionId) -> Option<RelationalCommitReceipt> {
        self.branch.commit_receipt_for_version(version).or_else(|| {
            self.performed
                .by_version(version)
                .map(|envelope| envelope.commit.clone())
        })
    }

    pub fn latest_patch_stream_position(self) -> Option<PatchStreamPosition> {
        self.branch
            .latest_recorded_patch_position()
            .max(self.performed.latest_patch())
    }

    pub fn latest_receipt(self) -> Option<RelationalCommitReceipt> {
        self.latest_patch_stream_position()
            .and_then(|position| {
                self.performed
                    .by_patch(position)
                    .map(|envelope| envelope.commit.clone())
                    .or_else(|| {
                        self.branch
                            .recorded_commit_at_patch_position(position)
                            .and_then(|commit| self.branch.commit_receipt(commit))
                    })
            })
            .or_else(|| self.branch.latest_commit_receipt())
    }

    pub fn canonical_envelope(self, commit: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.branch
            .commit_artifact(commit)
            .and_then(|artifact| artifact.resident_envelope())
            .or_else(|| self.performed.resident_envelope(commit))
    }

    pub fn demand_commit_envelope(
        self,
        execution: &ExecutionRequest<'_, '_>,
        commit: CommitId,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, RelationalExecutionDenialCause> {
        crate::execution::check_request(execution)?;
        let artifact = self.branch.try_commit_artifact(commit).map_err(|cause| {
            RelationalExecutionDenialCause::StorageUnavailable(
                crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Envelope(
                        PatchStreamPosition(0),
                    ),
                    cause,
                },
            )
        })?;
        match artifact {
            Some(artifact) => artifact
                .demand_envelope(execution, 1 << 30)
                .map(Some)
                .map_err(Into::into),
            None => Ok(self.performed.resident_envelope(commit)),
        }
    }
}
