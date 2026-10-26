use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

use super::HistorySubsystem;

impl HistorySubsystem {
    pub(crate) fn canonical_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.commit_artifact(commit_id)
            .and_then(|artifact| artifact.resident_envelope())
            .or_else(|| self.canonical_publication_routes.by_commit(commit_id))
    }

    pub(crate) fn canonical_envelope_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.by_version(version_id)
    }

    pub(crate) fn canonical_envelope_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.by_patch(position)
    }

    pub(crate) fn positioned_canonical_commit_at_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes
            .positioned_by_patch(position)
    }

    /// Performed route inventory for cold history and inspection consumers.
    pub(crate) fn performed_canonical_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.canonical_publication_routes.visible_envelopes()
    }

    pub(crate) fn latest_canonical_patch_route(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes.latest_visible_patch()
    }

    pub(crate) fn canonical_patch_routes_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes
            .visible_patches_after(after_position, max_commits)
    }

    pub(crate) fn latest_canonical_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.canonical_publication_routes
            .latest_patch_references(limit)
    }

    pub(crate) fn canonical_stream_position(
        &self,
        commit_id: CommitId,
    ) -> Option<PatchStreamPosition> {
        self.canonical_publication_routes.stream_position(commit_id)
    }

    pub(crate) fn positioned_canonical_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes
            .positioned_commit(commit_id)
    }

    pub(crate) fn positioned_canonical_commits_snapshot(
        &self,
    ) -> Vec<Arc<crate::history::data::PositionedCanonicalCommit>> {
        self.canonical_publication_routes.performed_snapshot()
    }

    pub(crate) fn canonical_checkpoint_gate(
        &self,
    ) -> Arc<super::RelationalCanonicalPublicationRoutes> {
        Arc::clone(&self.canonical_publication_routes)
    }

    pub(crate) fn advance_canonical_stream_floor(&self, floor: PatchStreamPosition) {
        self.canonical_publication_routes
            .advance_position_floor(floor);
    }

    pub(crate) fn install_recovered_canonical_route(
        &self,
        commit: Arc<crate::history::data::PositionedCanonicalCommit>,
    ) -> Result<(), &'static str> {
        self.canonical_publication_routes.install_recovered(commit)
    }

    pub(crate) fn rebuild_recovered_canonical_routes(
        &mut self,
        commits: impl IntoIterator<Item = Arc<crate::history::data::PositionedCanonicalCommit>>,
    ) -> Result<(), &'static str> {
        let routes = Arc::new(
            self.canonical_publication_routes
                .continuing_branch_transitions(),
        );
        for commit in commits {
            routes.install_recovered(commit)?;
        }
        self.canonical_publication_routes = routes;
        Ok(())
    }

    pub(crate) fn restore_branch_transition_coverage(
        &self,
        covered: crate::branch::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .restore_branch_transition_coverage(covered);
    }

    pub(crate) fn compact_branch_transitions_through(
        &self,
        covered: crate::branch::RelationalBranchTransitionSequence,
    ) {
        self.canonical_publication_routes
            .compact_branch_transitions_through(covered);
    }

    pub(crate) fn retained_branch_transitions(
        &self,
    ) -> (
        crate::branch::RelationalBranchTransitionSequence,
        Vec<Arc<crate::branch::PositionedBranchTransition>>,
    ) {
        self.canonical_publication_routes
            .retained_branch_transitions()
    }

    pub(crate) fn mark_publication_settled(&self, commit_id: CommitId) -> bool {
        self.canonical_publication_routes.mark_settled(commit_id)
    }

    pub(crate) fn publication_requires_settlement(&self, commit_id: CommitId) -> bool {
        self.canonical_publication_routes
            .requires_settlement(commit_id)
    }

    pub(crate) fn canonical_publication_reservation_counters(
        &self,
    ) -> super::RelationalPatchPositionReservationCounters {
        self.canonical_publication_routes.reservation_counters()
    }

    #[cfg(test)]
    pub(crate) fn pending_canonical_publication_route_count(&self) -> usize {
        self.canonical_publication_routes.pending_count()
    }

    #[cfg(test)]
    pub(crate) fn remove_canonical_publication_route_for_test(&self, commit_id: CommitId) -> bool {
        self.canonical_publication_routes.remove_for_test(commit_id)
    }
}
