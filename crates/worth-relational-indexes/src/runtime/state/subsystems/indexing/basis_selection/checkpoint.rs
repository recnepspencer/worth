//! Retained exact-root selection checkpoint capture and readmission.
use super::{IndexingState, PreparedIndexBasisSelection, Selections};

impl IndexingState {
    pub(crate) fn checkpoint_generation(
        &self,
        id: crate::contracts::indexes::data::definition::DerivedIndexGenerationId,
    ) -> Option<std::sync::Arc<crate::branch::indexes::data::DerivedIndexGeneration>> {
        self.generations.generation(id)
    }

    /// cost-lane: checkpoint. Select only retained roots; no timeline inventory.
    pub fn basis_checkpoint(
        &self,
        _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        retained: &crate::branch::history::retention::reclamation::RetainedIndexRoots,
    ) -> Vec<crate::contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint> {
        let mut bases = std::collections::BTreeMap::new();
        self.bases.visit_retained(retained, |basis| {
            bases.insert((basis.branch.clone(), basis.version), basis);
        });
        bases
            .into_values()
            .map(
                |basis| crate::contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint {
                    branch: basis.branch.clone(),
                    commit: basis.commit,
                    version: basis.version,
                    schema: basis.schema,
                    empty: basis.state.exact.empty.iter().copied().collect(),
                    bindings: basis
                        .state
                        .exact
                        .generations
                        .iter()
                        .map(|(index, generation)| (*index, generation.generation_id))
                        .collect(),
                },
            )
            .collect()
    }
    /// Retention consumes the requested roots' carried bindings directly; it
    /// never constructs a portable checkpoint merely to discover their ids.
    pub(in crate::internal_port::runtime::state::subsystems::indexing) fn visit_retained_bindings(
        &self,
        retained: &crate::branch::history::retention::reclamation::RetainedIndexRoots,
        mut visit: impl FnMut(crate::contracts::indexes::data::definition::DerivedIndexGenerationId),
    ) {
        self.bases.visit_retained(retained, |basis| {
            for generation in basis.state.exact.generations.values() {
                visit(generation.generation_id);
            }
        });
    }
    pub fn restore_basis_selection(
        &mut self,
        _lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        basis: &crate::contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint,
    ) -> Result<(), &'static str> {
        let mut entries = Selections::new();
        for (index, generation) in &basis.bindings {
            let generation = self
                .generations
                .generation(*generation)
                .ok_or("index basis generation is unavailable")?;
            if generation.index_id != *index
                || generation.applicability.schema_version != basis.schema
                || generation.applicability.version_id > basis.version
                || entries.contains_key(index)
            {
                return Err("index basis binding has inconsistent affinity");
            }
            entries.insert(*index, generation);
        }
        let mut empty = im::OrdSet::new();
        for index in &basis.empty {
            if entries.contains_key(index)
                || !self.catalog.definitions.contains_key(index)
                || empty.contains(index)
            {
                return Err("index empty selection is inconsistent");
            }
            empty.insert(*index);
        }
        let global = entries
            .iter()
            .filter(|(index, _)| self.catalog.globals.contains(index))
            .map(|(index, value)| (*index, value.clone()))
            .collect();
        let global_empty = empty
            .iter()
            .filter(|index| self.catalog.globals.contains(index))
            .copied()
            .collect();
        let restored = PreparedIndexBasisSelection {
            branch: basis.branch.clone(),
            commit: basis.commit,
            version: basis.version,
            schema: basis.schema,
            state: std::sync::Arc::new(super::super::IndexSelectionState {
                exact: super::super::selection_state::BasisBindings {
                    generations: entries,
                    empty,
                },
                global: super::super::selection_state::BasisBindings {
                    generations: global,
                    empty: global_empty,
                },
            }),
            globals: self.catalog.globals.clone(),
        };
        self.bases.install(restored);
        Ok(())
    }
}

impl super::BasisSelections {
    fn visit_retained<'basis>(
        &'basis self,
        retained: &crate::branch::history::retention::reclamation::RetainedIndexRoots,
        mut visit: impl FnMut(&'basis PreparedIndexBasisSelection),
    ) {
        for (branch, version, schema) in retained.live.iter().chain(&retained.retired) {
            if let Some(basis) = self
                .get(Some(branch), *version)
                .filter(|basis| basis.schema == *schema)
            {
                visit(basis);
            }
        }
        for (version, schema) in &retained.historical_versions {
            if let Some(selected) = self.versions.get(version) {
                for basis in selected
                    .branches
                    .values()
                    .filter(|basis| basis.schema == *schema)
                {
                    visit(basis);
                }
            }
        }
    }
}
