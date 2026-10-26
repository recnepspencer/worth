use std::sync::Arc;

use worth_execution::ExecutionAllocationPolicy;

use super::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial};
use crate::durability::access::DurabilityAccess;

impl DurabilityAccess<'_> {
    /// The runtime's retained performed canonical commits, in strictly
    /// increasing stream position, as artifacts. This reads the same
    /// performed publication stream checkpoints select from; it is not a
    /// second history store.
    ///
    /// The selection is fixed when this is called. Encoding is lazy: each
    /// item is framed only when pulled, under `policy`, so transient memory
    /// is one artifact at a time. Cost: O(retained commits) shared handles
    /// up front, then one artifact encode per pulled item.
    pub fn canonical_commit_artifacts<'scope, 'authority>(
        &self,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> RelationalCanonicalCommitArtifacts<'scope, 'authority> {
        let mut selected = std::collections::BTreeMap::new();
        for artifact in self.runtime.history.commit_artifacts() {
            let cell = artifact.envelope_residency();
            let position = cell
                .journal_index()
                .map(|entry| entry.position)
                .or_else(|| {
                    cell.positioned_if_resident()
                        .map(|commit| commit.position())
                });
            if let Some(position) = position {
                selected.insert(position, RetainedArtifact { position, cell });
            }
        }
        for commit in self.runtime.history.positioned_canonical_commits_snapshot() {
            if let Some(cell) = self
                .runtime
                .history
                .performed_envelope_residency(commit.commit.commit_id)
            {
                selected
                    .entry(commit.position())
                    .or_insert(RetainedArtifact {
                        position: commit.position(),
                        cell,
                    });
            }
        }
        let retained = selected.into_values().collect::<Vec<_>>();
        RelationalCanonicalCommitArtifacts {
            retained: retained.into_iter(),
            policy,
        }
    }
}

struct RetainedArtifact {
    position: crate::publication::patch::data::PatchStreamPosition,
    cell: Arc<crate::history::RelationalCommitEnvelopeResidency>,
}

/// Lazily framed artifacts for one fixed selection of retained commits, in
/// strictly increasing position order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalCanonicalCommitArtifacts<'scope, 'authority> {
    retained: std::vec::IntoIter<RetainedArtifact>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl RelationalCanonicalCommitArtifacts<'_, '_> {
    /// Select positions strictly above an acknowledged prefix before any
    /// journal body is demanded. Ordinary iterator skipping may pull and
    /// discard a body; this seek advances only immutable index entries.
    pub fn after_position(
        mut self,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) -> Self {
        let count = self
            .retained
            .as_slice()
            .iter()
            .take_while(|entry| entry.position <= position)
            .count();
        if count != 0 {
            let _ = self.retained.nth(count - 1);
        }
        self
    }
}

impl Iterator for RelationalCanonicalCommitArtifacts<'_, '_> {
    type Item =
        Result<RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial>;

    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // Skipping an immutable index prefix does not demand its bodies.
        self.retained
            .nth(n)
            .map(|commit| commit.cell.export_artifact(self.policy))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.retained.size_hint()
    }
}

impl ExactSizeIterator for RelationalCanonicalCommitArtifacts<'_, '_> {}

impl std::fmt::Debug for RelationalCanonicalCommitArtifacts<'_, '_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalCanonicalCommitArtifacts")
            .field("remaining", &self.retained.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "retained_export/skip_tests.rs"]
mod skip_tests;
