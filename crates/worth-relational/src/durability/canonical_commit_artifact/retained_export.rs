use std::sync::Arc;

use worth_execution::ExecutionRequest;

use super::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial};

struct RetainedArtifact {
    position: crate::publication::patch::data::PatchStreamPosition,
    cell: Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
}

/// Lazily framed artifacts for one fixed selection of retained commits, in
/// strictly increasing position order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalCanonicalCommitArtifacts<'scope, 'authority> {
    retained: std::vec::IntoIter<RetainedArtifact>,
    execution: ExecutionRequest<'scope, 'authority>,
}

impl<'scope, 'authority> RelationalCanonicalCommitArtifacts<'scope, 'authority> {
    pub(crate) fn new(
        selected: std::collections::BTreeMap<
            crate::publication::patch::data::PatchStreamPosition,
            Arc<crate::history::commit::RelationalCommitEnvelopeResidency>,
        >,
        execution: ExecutionRequest<'scope, 'authority>,
    ) -> Self {
        let retained = selected
            .into_iter()
            .map(|(position, cell)| RetainedArtifact { position, cell })
            .collect::<Vec<_>>();
        Self {
            retained: retained.into_iter(),
            execution,
        }
    }

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
        self.retained.nth(n).map(|commit| {
            commit
                .cell
                .export_artifact(&self.execution)
                .map(super::RelationalCanonicalCommitArtifact::from_verified)
        })
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
