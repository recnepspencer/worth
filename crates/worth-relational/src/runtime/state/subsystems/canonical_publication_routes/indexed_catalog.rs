//! Immutable indexed facts remain available while envelope bodies stay cold.
use super::{visible_envelope, RelationalCanonicalPublicationRoutes};
use crate::history::data::{
    CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit, RelationalCommitReceipt,
};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{atomic::Ordering, Arc};

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn install_checkpoint_catalog(
        &self,
        entries: Vec<Arc<crate::history::RelationalCommitArtifact>>,
    ) {
        *self
            .checkpoint_catalog
            .write()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(
            entries
                .into_iter()
                .map(|artifact| (artifact.commit_id(), artifact))
                .collect(),
        );
    }
    pub(crate) fn performed_envelope_residency(
        &self,
        id: CommitId,
    ) -> Option<Arc<crate::history::RelationalCommitEnvelopeResidency>> {
        self.by_commit
            .get(&id)
            .filter(|route| route.is_visible())
            .map(|route| route.envelope.clone())
    }
    pub(crate) fn demand_envelope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: CommitId,
    ) -> Result<
        Option<Arc<CanonicalCommitEnvelope>>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        let cell = self.performed_envelope_residency(id).or_else(|| {
            self.checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .get(&id)
                .map(|artifact| artifact.envelope_residency())
        });
        cell.map(|cell| cell.demand(execution, 1 << 30)).transpose()
    }
    pub(crate) fn indexed_receipt(&self, id: CommitId) -> Option<RelationalCommitReceipt> {
        if let Some(route) = self.by_commit.get(&id)
            && route.is_visible()
        {
            return Some(route.commit.clone());
        }
        self.checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
            .map(|artifact| artifact.receipt())
    }
    pub(crate) fn by_commit(&self, commit_id: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_commit.get(&commit_id).map(|entry| entry.clone())).or_else(|| {
            self.checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .get(&commit_id)
                .and_then(|artifact| artifact.resident_envelope())
        })
    }

    /// Includes reserved routes; bounds traversal through the visible subset.
    pub(crate) fn catalog_len_bound(&self) -> usize {
        self.by_commit.len()
            + self
                .checkpoint_catalog
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .len()
    }

    pub(crate) fn by_version(&self, version_id: VersionId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_version.get(&version_id).map(|entry| entry.clone()))
    }

    pub(crate) fn by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.envelope_at(position)
    }

    pub(crate) fn positioned_by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_at(position)
    }

    pub(crate) fn visible_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.visible_envelopes()
    }

    pub(crate) fn latest_visible_patch(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.performed_stream.latest()
    }

    pub(crate) fn visible_patches_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.performed_stream.after(after_position, max_commits)
    }

    pub(crate) fn latest_patch_references(
        &self,
        limit: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.performed_stream.latest_references(limit)
    }

    pub(crate) fn stream_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        let route = self
            .by_commit
            .get(&commit_id)
            .map(|entry| Arc::clone(entry.value()))?;
        route.is_visible().then(|| route.position()).flatten()
    }

    pub(crate) fn positioned_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned(commit_id)
    }

    pub(crate) fn requires_settlement(&self, commit_id: CommitId) -> bool {
        self.by_commit
            .get(&commit_id)
            .is_some_and(|route| !route.settled.load(Ordering::Acquire))
    }

    pub(crate) fn performed_snapshot(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_snapshot()
    }
}
