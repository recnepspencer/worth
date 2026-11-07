use crate::indexes::data::SelectedIndexReadWork;
use crate::indexes::data::{DerivedIndexId, SelectedIndexGenerationAdmissionStop};
use crate::mvcc::RelationalBranchObservation;

use super::{generation_catalog::navigation_work, IndexingSubsystem};

impl IndexingSubsystem {
    pub(crate) fn maintenance_inputs_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        branch: &crate::history::data::BranchId,
        commit: crate::history::data::CommitId,
        version: crate::identity::data::VersionId,
        before: Option<(
            crate::identity::data::VersionId,
            crate::schema::data::SchemaVersionId,
            crate::history::data::CommitId,
        )>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<(
            std::sync::Arc<crate::indexes::data::DerivedIndexDefinition>,
            Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>>,
            Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>>,
        )>,
        SelectedIndexGenerationAdmissionStop<Stop>,
    > {
        let state = self.state.read();
        let visits = navigation_work(state.catalog.definitions.len(), 1)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(definition) = state.catalog.definitions.get(&index) else {
            return Ok(None);
        };
        let scope = definition.branch_scoped.then_some(branch);
        prepare(
            navigation_work(state.bases.len(), branch.0.len().saturating_add(1))?,
            0,
        )
        .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let generation = if let Some(basis) = state.bases.get(scope, version) {
            state.generations.note_payload_selection();
            (basis.commit == commit)
                .then(|| basis.generation(index))
                .flatten()
        } else {
            state.generations.published_for_commit_admitted(
                index,
                scope,
                commit,
                version,
                &mut prepare,
            )?
        };
        let prior = if let Some((version, schema, commit)) = before {
            prepare(
                navigation_work(state.bases.len(), branch.0.len().saturating_add(1))?,
                0,
            )
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
            if let Some(basis) = state
                .bases
                .get(scope, version)
                .or_else(|| state.bases.get(None, version))
            {
                state.generations.note_payload_selection();
                (basis.schema == schema && basis.commit == commit)
                    .then(|| basis.generation(index))
                    .flatten()
            } else {
                state
                    .generations
                    .exact_admitted(index, scope, version, schema, &mut |units, bytes| {
                        let (SelectedIndexReadWork::Operation(units)
                        | SelectedIndexReadWork::OrderedNavigation(units)) = units;
                        prepare(units, bytes)
                    })?
                    .filter(|generation| generation.source_commit_id == commit)
            }
        } else {
            None
        };
        Ok(Some((std::sync::Arc::clone(definition), generation, prior)))
    }
    /// Check one selected published index under a single owner read lock.
    /// Each actual catalog descent and the branch-key copy is admitted before
    /// the corresponding read; this method never starts reconstruction.
    pub(crate) fn has_published_generation_for_observation_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        observation: &RelationalBranchObservation,
        mut prepare: impl FnMut(SelectedIndexReadWork, u64) -> Result<(), Stop>,
    ) -> Result<bool, SelectedIndexGenerationAdmissionStop<Stop>> {
        let Some(commit) = observation.commit_id() else {
            return Ok(false);
        };
        let state = self.state.read();
        let definition_work = navigation_work(state.catalog.definitions.len(), 1)?;
        prepare(SelectedIndexReadWork::OrderedNavigation(definition_work), 0)
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(definition) = state.catalog.definitions.get(&index) else {
            return Ok(false);
        };
        let branch = definition
            .branch_scoped
            .then_some(observation.identity().branch_id());
        prepare(
            SelectedIndexReadWork::OrderedNavigation(navigation_work(state.bases.len(), 1)?),
            0,
        )
        .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        if let Some(basis) = state.bases.get(branch, observation.version_id()) {
            prepare(
                SelectedIndexReadWork::OrderedNavigation(navigation_work(
                    state.catalog.definitions.len(),
                    1,
                )?),
                0,
            )
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
            return Ok(basis.commit == commit && basis.generation(index).is_some());
        }
        state.generations.has_published_for_commit_admitted(
            index,
            definition
                .branch_scoped
                .then_some(observation.identity().branch_id()),
            commit,
            observation.version_id(),
            &mut prepare,
        )
    }
}
