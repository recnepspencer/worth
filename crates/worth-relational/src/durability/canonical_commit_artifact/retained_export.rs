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
        let directory = self.runtime.durability.checkpoint_selection().snapshot();
        let mut retained = directory.as_ref().map_or_else(Vec::new, |directory| {
            directory
                .manifest
                .commits
                .iter()
                .map(|entry| RetainedArtifact::Cold {
                    entry: entry.clone(),
                    source: directory.source.clone(),
                })
                .collect()
        });
        retained.extend(
            self.runtime
                .history
                .positioned_canonical_commits_snapshot()
                .into_iter()
                .filter(|commit| {
                    directory.as_ref().is_none_or(|directory| {
                        commit.position() > directory.manifest.covered_position
                    })
                })
                .map(RetainedArtifact::Resident),
        );
        RelationalCanonicalCommitArtifacts {
            retained: retained.into_iter(),
            policy,
        }
    }
}

enum RetainedArtifact {
    Resident(Arc<PositionedCanonicalCommit>),
    Cold {
        entry: crate::history::RelationalCommitIndexEntry,
        source: Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
    },
}

/// Lazily framed artifacts for one fixed selection of retained commits, in
/// strictly increasing position order. A denial for one item does not end
/// the iteration; the caller decides whether to continue.
pub struct RelationalCanonicalCommitArtifacts<'scope, 'authority> {
    retained: std::vec::IntoIter<RetainedArtifact>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
}

impl Iterator for RelationalCanonicalCommitArtifacts<'_, '_> {
    type Item =
        Result<RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactEncodeDenial>;

    fn next(&mut self) -> Option<Self::Item> {
        self.retained.next().map(|commit| match commit {
            RetainedArtifact::Resident(commit) => {
                RelationalCanonicalCommitArtifact::from_positioned(&commit, self.policy)
            }
            RetainedArtifact::Cold { entry, source } => {
                self.policy.check_live()?;
                let failure = |cause| {
                    RelationalCanonicalCommitArtifactEncodeDenial::StorageUnavailable(
                        crate::storage::data::RelationalStorageUnavailable {
                            subject: crate::storage::data::RelationalStorageSubject::Envelope(
                                entry.position,
                            ),
                            cause,
                        },
                    )
                };
                let bytes = source
                    .read_commit(entry.position, 1 << 30)
                    .map_err(&failure)?;
                let artifact = RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes)
                    .map_err(|_| {
                        failure(crate::storage::data::RelationalStorageUnavailableCause::Damaged)
                    })?;
                if artifact.position() != entry.position
                    || artifact.commit_id() != entry.commit.commit_id
                    || artifact.payload_digest().as_bytes() != &entry.payload_digest
                {
                    return Err(failure(
                        crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch,
                    ));
                }
                self.policy.check_live()?;
                Ok(artifact)
            }
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
