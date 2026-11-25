use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;

use super::CommitEnvelopeSource;

pub(crate) use crate::publication::patch::data::stream_commit_ref::PatchStreamCommitRef;

pub(crate) trait PatchStreamSource: CommitEnvelopeSource {
    fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition>;
    fn commit_id_at_patch_stream_position(&self, position: PatchStreamPosition)
        -> Option<CommitId>;
    fn patch_stream_commits_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<PatchStreamCommitRef>;

    fn contains_patch_stream_position(&self, position: PatchStreamPosition) -> bool {
        self.commit_id_at_patch_stream_position(position).is_some()
    }

    fn commit_envelope_at_patch_stream_position(
        &self,
        position: PatchStreamPosition,
    ) -> Option<std::sync::Arc<CanonicalCommitEnvelope>> {
        self.commit_id_at_patch_stream_position(position)
            .and_then(|commit_id| self.resident_commit_envelope(commit_id))
    }
}

impl PatchStreamSource for RelationalRuntime {
    fn latest_patch_stream_position(&self) -> Option<PatchStreamPosition> {
        let projected = self.history.latest_recorded_patch_position();
        projected.max(
            self.history
                .latest_canonical_patch_route()
                .map(|(position, _)| position),
        )
    }

    fn commit_id_at_patch_stream_position(
        &self,
        position: PatchStreamPosition,
    ) -> Option<CommitId> {
        self.history
            .recorded_commit_at_patch_position(position)
            .or_else(|| {
                self.history
                    .canonical_envelope_at_patch(position)
                    .map(|envelope| envelope.commit.commit_id)
            })
    }

    fn patch_stream_commits_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<PatchStreamCommitRef> {
        crate::publication::canonical_history::patch_stream_commits_after(
            &self.history,
            after_position,
            max_commits,
        )
    }
}
