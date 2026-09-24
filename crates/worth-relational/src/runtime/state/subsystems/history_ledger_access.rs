use std::sync::Arc;

use crate::history::data::{CanonicalCommitEnvelope, CommitId, VersionNode};
use crate::history::{
    RelationalCommitArtifact, RelationalCommitCatalog, RelationalCommitCatalogEnvelopeAppendDenial,
    RelationalCommitIdentity,
};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

use super::{HistoryLedger, HistorySubsystem};

/// Ledger truth read through shared ownership.
///
/// Every accessor carries its answer out of the ledger lock: catalog artifacts
/// and canonical envelopes are already `Arc`, and the sidecar entries are small
/// enough to copy. No accessor lends a borrow into the lock, so no caller can
/// hold the ledger across storage, durability, or projection work.
impl HistorySubsystem {
    pub(crate) fn commit_artifact(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger.read().commit_catalog.get(commit_id).cloned()
    }

    pub(crate) fn latest_commit_artifact(&self) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger.read().commit_catalog.latest_artifact().cloned()
    }

    pub(crate) fn latest_commit_identity(&self) -> Option<RelationalCommitIdentity> {
        self.ledger.read().commit_catalog.latest_identity().cloned()
    }

    pub(crate) fn commit_artifact_for_version(
        &self,
        version_id: VersionId,
    ) -> Option<Arc<RelationalCommitArtifact>> {
        self.ledger
            .read()
            .commit_catalog
            .find_by_version(version_id)
            .cloned()
    }

    pub(crate) fn commit_artifacts(&self) -> Vec<Arc<RelationalCommitArtifact>> {
        self.ledger.read().commit_catalog.snapshot()
    }

    pub(crate) fn catalog_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.ledger
            .read()
            .commit_catalog
            .snapshot()
            .into_iter()
            .map(|artifact| Arc::clone(artifact.envelope()))
            .collect()
    }

    pub(crate) fn catalog_len(&self) -> usize {
        self.ledger.read().commit_catalog.len()
    }

    pub(crate) fn validate_catalog_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        self.ledger
            .read()
            .commit_catalog
            .validate_envelope(envelope)
    }

    pub(crate) fn validate_new_catalog_envelope(
        &self,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), RelationalCommitCatalogEnvelopeAppendDenial> {
        self.ledger
            .read()
            .commit_catalog
            .validate_new_envelope(envelope)
    }

    pub(crate) fn install_commit_catalog(&self, catalog: RelationalCommitCatalog) {
        self.ledger.write().commit_catalog = catalog;
    }

    pub(crate) fn recorded_commit_envelope(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.ledger.read().commit_envelopes.get(&commit_id).cloned()
    }

    pub(crate) fn recorded_commit_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.ledger
            .read()
            .commit_envelopes
            .values()
            .cloned()
            .collect()
    }

    pub(crate) fn recorded_commit_envelope_entries(
        &self,
    ) -> Vec<(CommitId, Arc<CanonicalCommitEnvelope>)> {
        self.ledger
            .read()
            .commit_envelopes
            .iter()
            .map(|(commit_id, envelope)| (*commit_id, Arc::clone(envelope)))
            .collect()
    }

    pub(crate) fn recorded_commit_envelope_count(&self) -> usize {
        self.ledger.read().commit_envelopes.len()
    }

    pub(crate) fn has_recorded_commit_envelope(&self, commit_id: CommitId) -> bool {
        self.ledger.read().commit_envelopes.contains_key(&commit_id)
    }

    pub(crate) fn latest_recorded_patch_position(&self) -> Option<PatchStreamPosition> {
        self.ledger
            .read()
            .patch_stream_index
            .last_key_value()
            .map(|(position, _)| *position)
    }

    pub(crate) fn recorded_commit_at_patch_position(
        &self,
        position: PatchStreamPosition,
    ) -> Option<CommitId> {
        self.ledger
            .read()
            .patch_stream_index
            .get(&position)
            .copied()
    }

    pub(crate) fn recorded_patch_positions_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        let start = after_position
            .map(std::ops::Bound::Excluded)
            .unwrap_or(std::ops::Bound::Unbounded);
        self.ledger
            .read()
            .patch_stream_index
            .range((start, std::ops::Bound::Unbounded))
            .take(max_commits)
            .map(|(position, commit_id)| (*position, *commit_id))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn recorded_patch_position_count(&self) -> usize {
        self.ledger.read().patch_stream_index.len()
    }

    #[cfg(test)]
    pub(crate) fn commit_graph_len(&self) -> usize {
        self.ledger.read().commit_graph.len()
    }

    pub(crate) fn insert_commit_graph_node(&self, commit_id: CommitId, node: VersionNode) {
        self.ledger.write().commit_graph.insert(commit_id, node);
    }

    /// The reserved commit/version identity floors, read together so a court
    /// cannot observe half of an advanced sequence.
    #[cfg(test)]
    pub(crate) fn reserved_sequence_floors(&self) -> (u64, u64) {
        let ledger = self.ledger.read();
        (ledger.next_commit_id, ledger.next_version_id)
    }

    /// Run one bounded, self-contained edit against the whole ledger.
    ///
    /// Callers must not re-enter the history subsystem from inside the closure;
    /// recovery paths read what they need first, build the replacement, and
    /// install it here.
    pub(crate) fn with_ledger_mut<T>(&self, edit: impl FnOnce(&mut HistoryLedger) -> T) -> T {
        edit(&mut self.ledger.write())
    }

    pub(crate) fn ledger_snapshot(&self) -> HistoryLedger {
        self.ledger.read().clone()
    }
}
