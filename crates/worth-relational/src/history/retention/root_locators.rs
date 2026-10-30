//! Secondary locators for physical roots by commit and exact version/schema.

use std::collections::BTreeSet;

use dashmap::{mapref::entry::Entry, DashMap};

use crate::branch::RelationalBranchRoot;
use crate::history::data::CommitId;
use crate::identity::data::VersionId;
use crate::schema::data::SchemaVersionId;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct RetiredRootLocators {
    commits: DashMap<CommitId, BTreeSet<(usize, u64)>>,
    bases: DashMap<(VersionId, SchemaVersionId), BTreeSet<(usize, u64)>>,
}

impl RetiredRootLocators {
    pub(super) fn register(&self, root: &RelationalBranchRoot, root_key: usize, generation: u64) {
        if let Some(envelope) = root.canonical_envelope() {
            self.register_locator(
                envelope.commit.commit_id,
                (
                    envelope.commit.version_id,
                    root.schema_authority().schema_version(),
                ),
                (root_key, generation),
            );
        }
    }

    fn register_locator(
        &self,
        commit: CommitId,
        basis: (VersionId, SchemaVersionId),
        locator: (usize, u64),
    ) {
        self.commits.entry(commit).or_default().insert(locator);
        self.bases.entry(basis).or_default().insert(locator);
    }

    /// Carry locators out of this index before entering the root inventory.
    pub(super) fn snapshot(&self, commit: CommitId) -> Vec<(usize, u64)> {
        self.commits
            .get(&commit)
            .map(|roots| roots.iter().copied().collect())
            .unwrap_or_default()
    }

    pub(super) fn snapshot_basis(
        &self,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Vec<(usize, u64)> {
        self.bases
            .get(&(version, schema))
            .map(|roots| roots.iter().copied().collect())
            .unwrap_or_default()
    }

    pub(super) fn remove(&self, root: &RelationalBranchRoot, root_key: usize, generation: u64) {
        let Some(envelope) = root.canonical_envelope() else {
            return;
        };
        self.remove_locator(
            envelope.commit.commit_id,
            (
                envelope.commit.version_id,
                root.schema_authority().schema_version(),
            ),
            (root_key, generation),
        );
    }

    fn remove_locator(
        &self,
        commit: CommitId,
        basis: (VersionId, SchemaVersionId),
        locator: (usize, u64),
    ) {
        if let Entry::Occupied(mut entry) = self.commits.entry(commit) {
            entry.get_mut().remove(&locator);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
        if let Entry::Occupied(mut entry) = self.bases.entry(basis) {
            entry.get_mut().remove(&locator);
            if entry.get().is_empty() {
                entry.remove();
            }
        }
    }
}
