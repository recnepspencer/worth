//! Retained exact-root selection checkpoint capture and readmission.
use super::{IndexingState, PreparedIndexBasisSelection, Selections};

impl IndexingState {
    pub(crate) fn checkpoint_generation(
        &self,
        id: crate::indexes::data::DerivedIndexGenerationId,
    ) -> Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>> {
        self.generations.generation(id)
    }

    /// cost-lane: checkpoint. Select only retained roots; no timeline inventory.
    pub(crate) fn basis_checkpoint(
        &self,
        retained: &crate::history::retention::RetainedIndexRoots,
    ) -> Vec<crate::indexes::data::IndexBasisCheckpoint> {
        let mut bases = std::collections::BTreeMap::new();
        for (branch, version, schema) in retained.live.iter().chain(&retained.retired) {
            if let Some(basis) = self
                .bases
                .get(Some(branch), *version)
                .filter(|basis| basis.schema == *schema)
            {
                bases.insert((basis.branch.clone(), basis.version), basis);
            }
        }
        for (version, schema) in &retained.historical_versions {
            // Historical custody is version-wide. Keep the concrete branch
            // selections too; the global view intentionally owns no scoped rows.
            if let Some(selected) = self.bases.versions.get(version) {
                for basis in selected
                    .branches
                    .values()
                    .filter(|basis| basis.schema == *schema)
                {
                    bases.insert((basis.branch.clone(), basis.version), basis);
                }
            }
        }
        bases
            .into_values()
            .map(|basis| crate::indexes::data::IndexBasisCheckpoint {
                branch: basis.branch.clone(),
                commit: basis.commit,
                version: basis.version,
                schema: basis.schema,
                empty: basis.state.empty.iter().copied().collect(),
                bindings: basis
                    .state
                    .entries
                    .iter()
                    .map(|(index, generation)| (*index, generation.generation_id))
                    .collect(),
            })
            .collect()
    }
    pub(crate) fn restore_basis_selection(
        &mut self,
        basis: &crate::indexes::data::IndexBasisCheckpoint,
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
                entries,
                empty,
                global,
                global_empty,
            }),
            globals: self.catalog.globals.clone(),
        };
        self.bases.install(restored);
        Ok(())
    }
}
