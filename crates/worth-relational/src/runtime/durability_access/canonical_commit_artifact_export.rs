use worth_relational_durability::internal_port::durability::canonical_commit_artifact::retained_export::RelationalCanonicalCommitArtifactsOwnerPort as _;
use super::DurabilityAccess;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifacts;
use worth_execution::ExecutionRequest;

impl DurabilityAccess<'_> {
    /// The runtime's retained performed canonical commits, in strictly
    /// increasing stream position, as artifacts. This reads the same
    /// performed publication stream checkpoints select from; it is not a
    /// second history store.
    ///
    /// The selection is fixed when called. New Memory publications verify and
    /// share their retained original frame when pulled, without encoding or a
    /// replacement byte allocation. Legacy and Journal paths keep their ordinary
    /// export behavior. Selection costs O(retained commits) shared metadata;
    /// verifying an original costs O(artifact bytes), with no new payload copy.
    pub fn canonical_commit_artifacts<'scope, 'authority>(
        &self,
        execution: &ExecutionRequest<'scope, 'authority>,
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
                selected.insert(position, cell);
            }
        }
        for commit in self.runtime.history.positioned_canonical_commits_snapshot() {
            if let Some(cell) = self
                .runtime
                .history
                .performed_envelope_residency(commit.commit.commit_id)
            {
                selected.entry(commit.position()).or_insert(cell);
            }
        }
        RelationalCanonicalCommitArtifacts::new(selected, *execution)
    }
}
