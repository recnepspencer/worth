use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;

use super::CommitEnvelopeSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PatchStreamCommitRef {
    pub position: PatchStreamPosition,
    pub commit_id: CommitId,
}

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
    ) -> Option<CanonicalCommitEnvelope> {
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
        let start = after_position
            .map(std::ops::Bound::Excluded)
            .unwrap_or(std::ops::Bound::Unbounded);
        let mut entries = self
            .history
            .recorded_patch_positions_after(after_position, max_commits)
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        entries.extend(
            self.history
                .canonical_patch_routes_after(after_position, max_commits),
        );
        entries
            .range((start, std::ops::Bound::Unbounded))
            .map(|(position, commit_id)| PatchStreamCommitRef {
                position: *position,
                commit_id: *commit_id,
            })
            .take(max_commits)
            .collect()
    }
}
