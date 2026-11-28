pub mod checkpoint_image;
pub mod checkpoint_inputs;
pub mod checkpoint_publication;
pub mod construction;
pub use checkpoint_inputs::{FrozenCheckpointIndexes, OriginalCheckpointIndexes};
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, FieldKey};

pub mod selection_state;
pub(crate) use selection_state::IndexSelectionState;
pub mod basis_selection;
pub(crate) use basis_selection::PreparedIndexBasisSelection;
pub mod admitted_definition;
pub use crate::branch::runtime::state::subsystems::indexing::catalog_coverage;
pub mod catalog_publication;
pub mod definition_catalog;
pub mod definition_preparation;
pub mod definition_publication;
pub(crate) use crate::branch::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage;
pub use catalog_publication::CapturedIndexDefinitionCatalog;
pub(crate) use definition_catalog::IndexDefinitionCatalog;
pub use definition_preparation::{
    PreparedRelationalIndexDefinitions, RelationalIndexDefinitionPreparation,
    RelationalIndexDefinitionPreparationDenial,
};
pub mod admitted_selection;
#[cfg(test)]
mod catalog_certification_tests;
pub mod generation_publication_gate;
#[cfg(test)]
pub mod identity_exhaustion_tests;
pub mod prepared_publication;
use generation_publication_gate::GenerationPublicationGate;
pub(crate) use prepared_publication::PreparedGenerationBatch;
pub mod discard;
pub mod generation_catalog;
mod catalog_lifecycle;

pub mod publication_retirement;
#[doc(hidden)]
pub mod query_generation_routing;
mod shared_state;
use generation_catalog::GenerationCatalog;
use shared_state::SharedIndexingState;

use crate::branch::indexes::data::{DerivedIndexArtifacts, DerivedIndexGeneration};
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::indexes::data::definition::{
    DerivedIndexDefinition, DerivedIndexGenerationId, DerivedIndexId,
};
use crate::contracts::schema::data::SchemaVersionId;
use worth_relational_storage::internal_port::storage::substrate::owned_state::RuntimeOwnedState;

/// Definitions and retained derived generations owned by the native index runtime.
///
/// Definitions and generations are held behind `Arc` so a reader can carry one
/// out of the subsystem lock without copying its entries and without retaining
/// the guard.
#[derive(Debug, Clone)]
pub struct IndexingState {
    catalog: Arc<IndexDefinitionCatalog>,
    generations: GenerationCatalog,
    bases: basis_selection::BasisSelections,
    pub(crate) next_index_id: u64,
    pub(crate) next_generation_id: u64,
}

impl IndexingState {
    pub fn retained_generations(
        &self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        retained: &crate::branch::history::retention::reclamation::RetainedIndexRoots,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let _ = lane;
        self.retained_catalog_generations(retained)
    }

    fn empty() -> Self {
        Self {
            catalog: Arc::new(IndexDefinitionCatalog::default()),
            generations: GenerationCatalog::default(),
            bases: basis_selection::BasisSelections::default(),
            next_index_id: 1,
            next_generation_id: 1,
        }
    }

    pub(crate) fn insert_definition(&mut self, definition: DerivedIndexDefinition) {
        self.next_index_id = self
            .next_index_id
            .max(definition.index_id.0.saturating_add(1));
        Arc::make_mut(&mut self.catalog).insert(definition);
    }

    pub fn definition(
        &self,
        index: DerivedIndexId,
    ) -> Option<&crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition> {
        self.catalog.definitions.get(&index)
    }

    pub fn restore_generation(&mut self, generation: DerivedIndexGeneration) {
        let id = generation.generation_id;
        self.restore_generation_payload(generation);
        self.record_basis_generation(id);
    }

    pub fn generation_frontier(&self) -> u64 {
        self.generations.generation_frontier()
    }

    pub fn restore_generation_frontier(&mut self, frontier: u64) {
        self.next_generation_id = self.next_generation_id.max(frontier);
        self.generations.restore_frontier(frontier);
    }
}

impl Default for IndexingState {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Default)]
pub struct IndexingSubsystem {
    state: SharedIndexingState,
    definition_publication: RuntimeOwnedState<()>,
    generation_publication: Arc<GenerationPublicationGate>,
}

/// Index catalog custody for change sealing and native publication preparation.
#[derive(Debug)]
pub struct IndexDefinitionReadBinding {
    state: SharedIndexingState,
    definition_publication: RuntimeOwnedState<()>,
    generation_publication: Arc<GenerationPublicationGate>,
}

impl Clone for IndexDefinitionReadBinding {
    fn clone(&self) -> Self {
        Self {
            state: self.state.share(),
            definition_publication: self.definition_publication.share(),
            generation_publication: Arc::clone(&self.generation_publication),
        }
    }
}

impl IndexDefinitionReadBinding {
    pub(crate) fn indexing_subsystem(&self) -> IndexingSubsystem {
        IndexingSubsystem {
            state: self.state.share(),
            definition_publication: self.definition_publication.share(),
            generation_publication: Arc::clone(&self.generation_publication),
        }
    }

    pub fn snapshot(&self) -> Arc<IndexDefinitionCatalog> {
        Arc::clone(&self.state.read().catalog)
    }
}

fn field_index_locator(
    kind: &crate::contracts::indexes::data::definition::DerivedIndexKind,
) -> Option<(AspectKey, FieldKey)> {
    match kind {
        crate::contracts::indexes::data::definition::DerivedIndexKind::EntityField {
            field_locator,
        }
        | crate::contracts::indexes::data::definition::DerivedIndexKind::RelationField {
            field_locator,
        } => match field_locator.field_path().fields() {
            [field] => Some((field_locator.aspect().aspect_key().clone(), field.clone())),
            _ => None,
        },
        _ => None,
    }
}

impl IndexingSubsystem {
    pub fn definition_read_binding(&self) -> IndexDefinitionReadBinding {
        IndexDefinitionReadBinding {
            state: self.state.share(),
            definition_publication: self.definition_publication.share(),
            generation_publication: Arc::clone(&self.generation_publication),
        }
    }
    pub fn reclaim_except_versions(
        &self,
        retained: &crate::branch::history::retention::reclamation::RetainedIndexRoots,
    ) -> usize {
        self.state.write().reclaim_catalog_generations(retained)
    }

    pub fn snapshot(&self) -> IndexingState {
        self.state.read().clone()
    }

    pub fn definition(
        &self,
        index_id: DerivedIndexId,
    ) -> Option<crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition> {
        self.state
            .read()
            .catalog
            .definitions
            .get(&index_id)
            .cloned()
    }

    pub fn with_definitions<R>(
        &self,
        read: impl FnOnce(
            &BTreeMap<
                DerivedIndexId,
                crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition,
            >,
        ) -> R,
    ) -> R {
        read(&self.state.read().catalog.definitions)
    }

    pub fn definitions(
        &self,
    ) -> Vec<crate::branch::indexes::retained_definition::RetainedDerivedIndexDefinition> {
        self.state
            .read()
            .catalog
            .definitions
            .values()
            .cloned()
            .collect()
    }

    pub fn latest_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.latest(index, branch)
    }

    pub fn exact_generation(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        version: VersionId,
        schema: SchemaVersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        let state = self.state.read();
        if let Some(basis) = state.bases.get(branch, version) {
            state.generations.note_payload_selection();
            return (basis.schema == schema)
                .then(|| basis.generation(index))
                .flatten();
        }
        state.generations.exact(index, branch, version, schema)
    }

    pub fn published_generation_for_commit(
        &self,
        index: DerivedIndexId,
        branch: Option<&BranchId>,
        commit: CommitId,
        version: VersionId,
    ) -> Option<Arc<DerivedIndexGeneration>> {
        let state = self.state.read();
        if let Some(basis) = state.bases.get(branch, version) {
            state.generations.note_payload_selection();
            return (basis.commit == commit)
                .then(|| basis.generation(index))
                .flatten();
        }
        state
            .generations
            .published_for_commit(index, branch, commit, version)
    }

    pub fn all_generations(&self) -> Vec<Arc<DerivedIndexGeneration>> {
        self.state.read().generations.all()
    }

    pub fn generation_selection_counters(
        &self,
    ) -> crate::contracts::indexes::data::generation_selection_counters::DerivedIndexSelectionCounters
    {
        self.state.read().generations.selection_counters()
    }

    /// Reserve the next generation identity without publishing anything.
    pub fn next_generation_id(&self) -> Option<u64> {
        let mut state = self.state.write();
        let generation_id = state.next_generation_id;
        state.next_generation_id = state.next_generation_id.checked_add(1)?;
        Some(generation_id)
    }

    /// Reserve exact generation identities before the candidate can publish.
    pub fn reserve_generation_ids(&self, count: usize) -> Option<Vec<DerivedIndexGenerationId>> {
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

    /// Install a generation carried by a canonical envelope, replacing any
    /// earlier copy of the same generation identity.
    pub fn restore_generation(&self, generation: DerivedIndexGeneration) {
        let _generations = self.generation_publication.enter_uncontrolled();
        self.state.write().restore_generation(generation);
    }

    pub fn derived_artifacts_for_commit(&self, commit_id: CommitId) -> DerivedIndexArtifacts {
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

    /// Adversarial courts corrupt the newest generation of an index in place to
    /// prove that index-backed reads still deny rather than answer from it.
    pub fn corrupt_latest_generation_for_test(
        &self,
        index_id: DerivedIndexId,
        corrupt: impl FnOnce(&mut DerivedIndexGeneration),
    ) {
        let _generations = self.generation_publication.enter_uncontrolled();
        let mut state = self.state.write();
        let mut generation = state
            .generations
            .latest(index_id, None)
            .expect("court installs the generation it corrupts");
        corrupt(Arc::make_mut(&mut generation));
        state.restore_generation(generation.as_ref().clone());
    }
}

mod unique_field_probe;
