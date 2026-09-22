//! Rebuildable selection indexes over retained derived generations.
mod scope;
mod selection_work;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{DerivedIndexGeneration, DerivedIndexGenerationId, DerivedIndexId};
use crate::schema::data::SchemaVersionId;

use scope::GenerationScope;

/// Generation payloads have one owner. Selection tables carry identities only;
/// restoring a replacement removes every old binding before installing the new one.
#[derive(Debug, Clone, Default)]
pub(super) struct GenerationCatalog {
    entries: BTreeMap<DerivedIndexGenerationId, Arc<DerivedIndexGeneration>>,
    scopes: BTreeMap<(DerivedIndexId, Option<BranchId>), GenerationScope>,
    commits: BTreeMap<CommitId, BTreeSet<DerivedIndexGenerationId>>,
    versions: BTreeMap<VersionId, BTreeSet<DerivedIndexGenerationId>>,
    work: selection_work::SelectionWork,
}

impl GenerationCatalog {
    pub(super) fn publish(&mut self, generation: DerivedIndexGeneration) {
        assert!(
            !self.entries.contains_key(&generation.generation_id),
            "native index publication cannot replace a retained generation identity"
        );
        self.insert(generation);
    }

    pub(super) fn insert(&mut self, generation: DerivedIndexGeneration) {
        if let Some(previous) = self.entries.remove(&generation.generation_id) {
            self.remove_bindings(&previous);
        }
        for branch in [None, Some(generation.applicability.branch_id.clone())] {
            self.scopes
                .entry((generation.index_id, branch))
                .or_default()
                .insert(&generation);
        }
        self.commits
            .entry(generation.source_commit_id)
            .or_default()
            .insert(generation.generation_id);
        self.versions
            .entry(generation.applicability.version_id)
            .or_default()
            .insert(generation.generation_id);
        self.entries
            .insert(generation.generation_id, Arc::new(generation));
    }

    fn remove_bindings(&mut self, generation: &DerivedIndexGeneration) {
        for branch in [None, Some(generation.applicability.branch_id.clone())] {
            let key = (generation.index_id, branch);
            if let Some(scope) = self.scopes.get_mut(&key) {
                scope.remove(generation);
                if scope.is_empty() {
                    self.scopes.remove(&key);
                }
            }
        }
        remove_member(
            &mut self.commits,
            &generation.source_commit_id,
            generation.generation_id,
        );
        remove_member(
            &mut self.versions,
            &generation.applicability.version_id,
            generation.generation_id,
        );
    }

    pub(super) fn generation(
        &self,
        id: DerivedIndexGenerationId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        let generation = self.entries.get(&id)?;
        self.work.read_payload();
        Some(Arc::clone(generation))
    }

    pub(super) fn latest(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.generation(self.scope(index, branch)?.latest()?)
    }

    pub(super) fn exact(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.generation(self.scope(index, branch)?.exact(version, schema)?)
    }

    pub(super) fn published_for_commit(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        commit: CommitId,
        version: VersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.generation(
            self.scope(index, branch)?
                .published_for_commit(commit, version)?,
        )
    }

    /// Preserve routing precedence: published, branch, exact version, exact
    /// schema, then version and generation identity. At most one payload is read.
    pub(super) fn candidate(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        let all = self.scope(index, None)?;
        let published = all.has_published();
        let selected = self
            .scope(index, branch)
            .filter(|scope| scope.has_status(published))
            .unwrap_or(all);
        self.generation(selected.candidate(published, version, schema)?)
    }

    fn scope(&self, index: DerivedIndexId, branch: Option<&BranchId>) -> Option<&GenerationScope> {
        self.scopes.get(&(index, branch.cloned()))
    }

    pub(super) fn for_commit(&self, commit: CommitId) -> Vec<Arc<DerivedIndexGeneration>> {
        self.commits
            .get(&commit)
            .into_iter()
            .flatten()
            .filter_map(|id| self.generation(*id))
            .collect()
    }

    pub(super) fn any_at_or_before(&self, version: VersionId) -> bool {
        self.versions
            .first_key_value()
            .is_some_and(|(first, _)| *first <= version)
    }

    /// Explicit cold inventory for checkpointing and forensic inspection.
    pub(super) fn all(&self) -> Vec<Arc<DerivedIndexGeneration>> {
        self.work.enumerate_inventory(self.entries.len());
        let mut generations = self.entries.values().cloned().collect::<Vec<_>>();
        generations.sort_by_key(|generation| (generation.index_id, generation.generation_id));
        generations
    }

    pub(super) fn selection_counters(&self) -> crate::indexes::data::DerivedIndexSelectionCounters {
        self.work.snapshot()
    }
}

fn remove_member<K: Ord>(
    map: &mut BTreeMap<K, BTreeSet<DerivedIndexGenerationId>>,
    key: &K,
    id: DerivedIndexGenerationId,
) {
    if let Some(ids) = map.get_mut(key) {
        ids.remove(&id);
        if ids.is_empty() {
            map.remove(key);
        }
    }
}
