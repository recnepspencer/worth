use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectKey, FieldKey};

mod admitted_selection;
pub(crate) use admitted_selection::ExactLookupInputs;
mod generation_catalog;

use generation_catalog::GenerationCatalog;

use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::indexes::data::{
    DerivedIndexArtifacts, DerivedIndexDefinition, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId,
};
use crate::runtime::state::subsystems::{RuntimeOwnedState, RuntimeSubsystem};
use crate::schema::data::SchemaVersionId;
use crate::storage::data::AuthoritativeFieldComparisonKey;

/// Every entity that currently carries a tracked unique aspect field value.
pub(crate) type UniqueEntityAspectFieldIndex = BTreeMap<
    AspectFieldLocator,
    BTreeMap<AuthoritativeFieldComparisonKey, BTreeSet<crate::identity::data::EntityId>>,
>;

/// Definitions and retained derived generations owned by the native index runtime.
///
/// Definitions and generations are held behind `Arc` so a reader can carry one
/// out of the subsystem lock without copying its entries and without retaining
/// the guard.
#[derive(Debug, Clone)]
pub(crate) struct IndexingState {
    pub(crate) definitions: BTreeMap<DerivedIndexId, Arc<DerivedIndexDefinition>>,
    field_definitions: BTreeMap<AspectKey, BTreeMap<FieldKey, Vec<Arc<DerivedIndexDefinition>>>>,
    generations: GenerationCatalog,
    pub(crate) entity_unique_aspect_field_index: UniqueEntityAspectFieldIndex,
    pub(crate) next_index_id: u64,
    pub(crate) next_generation_id: u64,
}

impl IndexingState {
    fn empty() -> Self {
        Self {
            definitions: BTreeMap::new(),
            field_definitions: BTreeMap::new(),
            generations: GenerationCatalog::default(),
            entity_unique_aspect_field_index: BTreeMap::new(),
            next_index_id: 1,
            next_generation_id: 1,
        }
    }

    pub(crate) fn insert_definition(&mut self, definition: DerivedIndexDefinition) {
        self.next_index_id = self
            .next_index_id
            .max(definition.index_id.0.saturating_add(1));
        if let Some(previous) = self.definitions.remove(&definition.index_id) {
            if let Some((aspect, field)) = field_index_locator(&previous.kind) {
                if let Some(fields) = self.field_definitions.get_mut(&aspect) {
                    if let Some(definitions) = fields.get_mut(&field) {
                        definitions.retain(|entry| entry.index_id != previous.index_id);
                        if definitions.is_empty() {
                            fields.remove(&field);
                        }
                    }
                    if fields.is_empty() {
                        self.field_definitions.remove(&aspect);
                    }
                }
            }
        }
        let definition = Arc::new(definition);
        if let Some((aspect, field)) = field_index_locator(&definition.kind) {
            self.field_definitions
                .entry(aspect)
                .or_default()
                .entry(field)
                .or_default()
                .push(Arc::clone(&definition));
        }
        self.definitions.insert(definition.index_id, definition);
    }

    pub(crate) fn restore_generation(&mut self, generation: DerivedIndexGeneration) {
        self.next_generation_id = self
            .next_generation_id
            .max(generation.generation_id.0.saturating_add(1));
        self.generations.insert(generation);
    }
}

impl Default for IndexingState {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Default)]
pub(crate) struct IndexingSubsystem {
    state: RuntimeOwnedState<IndexingState>,
}

/// Read-only handle to installed definitions used while sealing commit changes.
#[derive(Debug)]
pub(crate) struct IndexDefinitionReadBinding {
    state: RuntimeOwnedState<IndexingState>,
}

impl Clone for IndexDefinitionReadBinding {
    fn clone(&self) -> Self {
        Self {
            state: self.state.share(),
        }
    }
}

impl IndexDefinitionReadBinding {
    pub(crate) fn with_field<R>(
        &self,
        aspect: &AspectKey,
        field: &FieldKey,
        read: impl FnOnce(&[Arc<DerivedIndexDefinition>]) -> R,
    ) -> R {
        let state = self.state.read();
        read(
            state
                .field_definitions
                .get(aspect)
                .and_then(|fields| fields.get(field))
                .map_or(&[], Vec::as_slice),
        )
    }
}

fn field_index_locator(
    kind: &crate::indexes::data::DerivedIndexKind,
) -> Option<(AspectKey, FieldKey)> {
    match kind {
        crate::indexes::data::DerivedIndexKind::EntityField { field_locator }
        | crate::indexes::data::DerivedIndexKind::RelationField { field_locator } => {
            match field_locator.field_path().fields() {
                [field] => Some((field_locator.aspect().aspect_key().clone(), field.clone())),
                _ => None,
            }
        }
        _ => None,
    }
}

impl IndexingSubsystem {
    pub(crate) fn definition_read_binding(&self) -> IndexDefinitionReadBinding {
        IndexDefinitionReadBinding {
            state: self.state.share(),
        }
    }
    pub(crate) fn reclaim_except_versions(
        &self,
        retained: &crate::history::retention::RetainedIndexRoots,
    ) -> usize {
        let mut state = self.state.write();
        let global_indexes = state
            .definitions
            .values()
            .filter(|definition| !definition.branch_scoped)
            .map(|definition| definition.index_id)
            .collect();
        state
            .generations
            .reclaim_except_versions(retained, &global_indexes)
    }

    pub(crate) fn retained_generations(
        &self,
        retained: &crate::history::retention::RetainedIndexRoots,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let state = self.state.read();
        let global_indexes = state
            .definitions
            .values()
            .filter(|definition| !definition.branch_scoped)
            .map(|definition| definition.index_id)
            .collect();
        state.generations.retained(retained, &global_indexes)
    }

    /// Replace the whole subsystem, for checkpoint restore.
    pub(crate) fn install(&self, state: IndexingState) {
        *self.state.write() = state;
    }

    pub(crate) fn snapshot(&self) -> IndexingState {
        self.state.read().clone()
    }

    pub(crate) fn definition(
        &self,
        index_id: DerivedIndexId,
    ) -> Option<Arc<DerivedIndexDefinition>> {
        self.state.read().definitions.get(&index_id).map(Arc::clone)
    }

    pub(crate) fn definitions(&self) -> Vec<Arc<DerivedIndexDefinition>> {
        self.state
            .read()
            .definitions
            .values()
            .map(Arc::clone)
            .collect()
    }

    pub(crate) fn generation(
        &self,
        id: DerivedIndexGenerationId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.generation(id)
    }

    pub(crate) fn latest_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.latest(index, branch)
    }

    pub(crate) fn exact_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state
            .read()
            .generations
            .exact(index, branch, version, schema)
    }

    pub(crate) fn published_generation_for_commit(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        commit: CommitId,
        version: VersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state
            .read()
            .generations
            .published_for_commit(index, branch, commit, version)
    }

    pub(crate) fn candidate_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state
            .read()
            .generations
            .candidate(index, branch, version, schema)
    }

    pub(crate) fn any_generation_at_or_before(&self, version: VersionId) -> bool {
        self.state.read().generations.any_at_or_before(version)
    }

    pub(crate) fn all_generations(&self) -> Vec<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.all()
    }

    pub(crate) fn generation_selection_counters(
        &self,
    ) -> crate::indexes::data::DerivedIndexSelectionCounters {
        self.state.read().generations.selection_counters()
    }

    /// Allocate the next definition identity and record the definition.
    pub(crate) fn register_definition(
        &self,
        mut definition: DerivedIndexDefinition,
    ) -> DerivedIndexDefinition {
        let mut state = self.state.write();
        definition.index_id = DerivedIndexId(state.next_index_id);
        state.next_index_id = state
            .next_index_id
            .checked_add(1)
            .expect("derived index identity exhausted");
        state.insert_definition(definition.clone());
        definition
    }

    /// Reserve the next generation identity without publishing anything.
    pub(crate) fn next_generation_id(&self) -> u64 {
        let mut state = self.state.write();
        let generation_id = state.next_generation_id;
        state.next_generation_id = state
            .next_generation_id
            .checked_add(1)
            .expect("derived generation identity exhausted");
        generation_id
    }

    /// Reserve exact generation identities before the candidate can publish.
    pub(crate) fn reserve_generation_ids(
        &self,
        count: usize,
    ) -> Option<Vec<DerivedIndexGenerationId>> {
        let mut state = self.state.write();
        let end = state
            .next_generation_id
            .checked_add(u64::try_from(count).ok()?)?;
        let mut ids = Vec::new();
        ids.try_reserve_exact(count).ok()?;
        ids.extend((state.next_generation_id..end).map(DerivedIndexGenerationId));
        state.next_generation_id = end;
        Some(ids)
    }

    pub(crate) fn publish_generation(&self, generation: DerivedIndexGeneration) {
        self.state.write().generations.publish(generation);
    }

    /// Install a generation carried by a canonical envelope, replacing any
    /// earlier copy of the same generation identity.
    pub(crate) fn restore_generation(&self, generation: DerivedIndexGeneration) {
        self.state.write().restore_generation(generation);
    }

    pub(crate) fn derived_artifacts_for_commit(
        &self,
        commit_id: CommitId,
    ) -> DerivedIndexArtifacts {
        DerivedIndexArtifacts::new(
            self.state
                .read()
                .generations
                .for_commit(commit_id)
                .into_iter()
                .map(|generation| generation.as_ref().clone())
                .collect(),
        )
    }

    /// Read the unique aspect field index without letting the guard escape.
    #[cfg(test)]
    pub(crate) fn with_unique_index<R>(
        &self,
        read: impl FnOnce(&UniqueEntityAspectFieldIndex) -> R,
    ) -> R {
        read(&self.state.read().entity_unique_aspect_field_index)
    }

    pub(crate) fn with_unique_index_mut<R>(
        &self,
        write: impl FnOnce(&mut UniqueEntityAspectFieldIndex) -> R,
    ) -> R {
        write(&mut self.state.write().entity_unique_aspect_field_index)
    }

    /// Adversarial courts corrupt the newest generation of an index in place to
    /// prove that index-backed reads still deny rather than answer from it.
    #[cfg(test)]
    pub(crate) fn corrupt_latest_generation(
        &self,
        index_id: DerivedIndexId,
        corrupt: impl FnOnce(&mut DerivedIndexGeneration),
    ) {
        let mut state = self.state.write();
        let mut generation = state
            .generations
            .latest(index_id, None)
            .expect("court installs the generation it corrupts");
        corrupt(Arc::make_mut(&mut generation));
        state.generations.insert(generation.as_ref().clone());
    }

    pub(crate) fn clear_unique_index(&self) {
        self.state.write().entity_unique_aspect_field_index.clear();
    }
}

impl RuntimeSubsystem for IndexingSubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self {
            state: RuntimeOwnedState::new(IndexingState::empty()),
        }
    }

    fn fork(&self) -> Self {
        Self {
            state: RuntimeOwnedState::new(self.snapshot()),
        }
    }
}
