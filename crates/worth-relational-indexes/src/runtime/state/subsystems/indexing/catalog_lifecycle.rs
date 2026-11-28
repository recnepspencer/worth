//! The index state coordinates generation payloads and their exact-root bindings.
use super::{basis_selection::PreparedIndexBasisSelection, generation_catalog, IndexingState};
use crate::branch::{
    history::{
        data::canonical_commit_envelope::CanonicalCommitEnvelope,
        retention::reclamation::RetainedIndexRoots,
    },
    indexes::data::DerivedIndexGeneration,
    runtime::state::subsystems::history::HistorySubsystem,
};
use crate::contracts::indexes::data::definition::DerivedIndexId;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

impl IndexingState {
    pub(super) fn retained_catalog_generations(
        &self,
        retained: &RetainedIndexRoots,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        let mut selected: BTreeMap<_, _> = self
            .generations
            .retained(retained, &self.global_indexes())
            .into_iter()
            .map(|generation| (generation.generation_id, generation))
            .collect();
        self.visit_retained_bindings(retained, |id| {
            if let Some(generation) = self.generations.generation(id) {
                selected.insert(id, generation);
            }
        });
        selected.into_values().collect()
    }

    pub(super) fn reclaim_catalog_generations(&mut self, retained: &RetainedIndexRoots) -> usize {
        let global_indexes = self.global_indexes();
        self.bases.reclaim(retained);
        let mut carried = BTreeSet::new();
        self.visit_retained_bindings(retained, |id| {
            carried.insert(id);
        });
        self.generations
            .reclaim_except_versions(retained, &global_indexes, &carried)
    }

    pub(super) fn discard_catalog_generations(&mut self, index: DerivedIndexId) -> usize {
        let removed = self.generations.discard_index(index);
        self.bases.discard(index);
        removed
    }

    pub(super) fn retire_catalog_after_publication(
        &mut self,
        history: &HistorySubsystem,
        publication: &CanonicalCommitEnvelope,
        maximum_older_visits: usize,
    ) -> Vec<Arc<DerivedIndexGeneration>> {
        self.bases
            .retire(maximum_older_visits, |branch, version, schema| {
                history.index_basis_is_held(version, schema, branch)
            });
        let IndexingState {
            catalog,
            generations,
            ..
        } = self;
        generations.retire_after_publication(publication, maximum_older_visits, |generation| {
            let global = catalog
                .definitions
                .get(&generation.index_id)
                .is_some_and(|definition| !definition.branch_scoped);
            (
                global,
                history.index_generation_basis_is_held(generation, !global),
                history.branch_cell(&generation.source_branch_id).is_some(),
            )
        })
    }

    pub(super) fn publication_quote<'batch>(
        &self,
        generations: &'batch [DerivedIndexGeneration],
        basis: Option<&PreparedIndexBasisSelection>,
    ) -> Option<(
        generation_catalog::admitted_publication::DeferredGenerationPublicationQuote<'batch>,
        (u64, u64),
    )> {
        Some((
            self.generations.deferred_quote(generations)?,
            self.bases.prepared_publication_quote(generations, basis)?,
        ))
    }

    fn global_indexes(&self) -> BTreeSet<DerivedIndexId> {
        self.catalog.globals.iter().copied().collect()
    }
}
