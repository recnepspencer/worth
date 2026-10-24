use std::sync::Arc;

use worth_execution::ExecutionAllocationPolicy;

use super::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial};
use crate::durability::access::DurabilityAccess;
use crate::history::data::PositionedCanonicalCommit;

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
        RelationalCanonicalCommitArtifacts {
            retained: self
                .runtime
                .history
                .positioned_canonical_commits_snapshot()
                .into_iter(),
            policy,
        }
    }
}

/// Lazily framed artifacts for one fixed selection of retained commits, in
/// strictly increasing position order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalCanonicalCommitArtifacts<'scope, 'authority> {
    retained: std::vec::IntoIter<Arc<PositionedCanonicalCommit>>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl Iterator for RelationalCanonicalCommitArtifacts<'_, '_> {
    type Item =
        Result<RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial>;

    fn next(&mut self) -> Option<Self::Item> {
        self.retained
            .next()
            .map(|commit| RelationalCanonicalCommitArtifact::from_positioned(&commit, self.policy))
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
