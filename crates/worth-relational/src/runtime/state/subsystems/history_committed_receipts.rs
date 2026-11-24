//! Immutable receipt selection across settled records and exact performed routes.
use super::HistorySubsystem;
use crate::history::data::RelationalCommitReceipt;
use crate::publication::patch::data::PatchStreamPosition;

impl HistorySubsystem {
    pub(crate) fn historical_latest_commit(&self) -> Option<RelationalCommitReceipt> {
        self.latest_patch_stream_position()
            .and_then(|position| {
                self.canonical_envelope_at_patch(position)
                    .map(|envelope| envelope.commit.clone())
                    .or_else(|| {
                        self.recorded_commit_at_patch_position(position)
                            .and_then(|commit_id| self.commit_artifact(commit_id))
                            .map(|artifact| artifact.receipt())
                    })
            })
            .or_else(|| {
                self.latest_commit_artifact()
                    .map(|artifact| artifact.receipt())
            })
    }

    pub(crate) fn immutable_commit_receipt_for_version(
        &self,
        version_id: crate::identity::data::VersionId,
    ) -> Option<RelationalCommitReceipt> {
        self.commit_artifact_for_version(version_id)
            .map(|artifact| artifact.receipt())
            .or_else(|| {
                self.canonical_envelope_for_version(version_id)
                    .map(|envelope| envelope.commit.clone())
            })
    }

    pub(crate) fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        let projected = self.latest_recorded_patch_position();
        projected.max(
            self.latest_canonical_patch_route()
                .map(|(position, _)| position),
        )
    }
}
