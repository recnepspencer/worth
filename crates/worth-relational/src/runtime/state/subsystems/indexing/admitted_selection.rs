use crate::indexes::data::{DerivedIndexId, SelectedIndexGenerationAdmissionStop};
use crate::mvcc::RelationalBranchObservation;

use super::{generation_catalog::navigation_work, IndexingSubsystem};

pub(crate) enum ExactLookupInputs {
    MissingDefinition,
    WrongKind,
    MissingGeneration,
    Ready {
        definition: std::sync::Arc<crate::indexes::data::DerivedIndexDefinition>,
        generation: std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>,
    },
}

impl IndexingSubsystem {
    pub(crate) fn exact_lookup_inputs_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        locator: &worth_foundational::facade::AspectFieldLocator,
        branch: &crate::history::data::BranchId,
        version: crate::identity::data::VersionId,
        schema: crate::schema::data::SchemaVersionId,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<ExactLookupInputs, SelectedIndexGenerationAdmissionStop<Stop>> {
        let state = self.state.read();
        let work = navigation_work(state.definitions.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(work, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(definition) = state.definitions.get(&index) else {
            return Ok(ExactLookupInputs::MissingDefinition);
        };
        let crate::indexes::data::DerivedIndexKind::EntityField { field_locator } =
            &definition.kind
        else {
            return Ok(ExactLookupInputs::WrongKind);
        };
        let path_visits = locator
            .field_path()
            .fields()
            .len()
            .checked_add(field_locator.field_path().fields().len())
            .and_then(|visits| u64::try_from(visits).ok())
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(path_visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let left = locator.aspect().aspect_key().as_str().len();
        let right = field_locator.aspect().aspect_key().as_str().len();
        let path = locator
            .field_path()
            .fields()
            .iter()
            .chain(field_locator.field_path().fields())
            .try_fold(0_usize, |total, field| {
                total.checked_add(field.as_str().len())
            })
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        let work = left
            .max(right)
            .checked_add(path)
            .and_then(|work| {
                work.checked_add(
                    locator
                        .field_path()
                        .fields()
                        .len()
                        .max(field_locator.field_path().fields().len()),
                )
            })
            .and_then(|work| work.checked_add(2))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(work, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        if field_locator != locator {
            return Ok(ExactLookupInputs::WrongKind);
        }
        let generation = state.generations.exact_admitted(
            index,
            definition.branch_scoped.then_some(branch),
            version,
            schema,
            &mut prepare,
        )?;
        Ok(match generation {
            Some(generation) => ExactLookupInputs::Ready {
                definition: std::sync::Arc::clone(definition),
                generation,
            },
            None => ExactLookupInputs::MissingGeneration,
        })
    }

    pub(crate) fn publish_generations_admitted<Stop>(
        &self,
        generations: &[crate::indexes::data::DerivedIndexGeneration],
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        self.state
            .write()
            .generations
            .publish_batch_admitted(generations, &mut prepare)
    }
    pub(crate) fn field_maintenance_inputs_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        branch: &crate::history::data::BranchId,
        commit: crate::history::data::CommitId,
        version: crate::identity::data::VersionId,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        Option<(
            std::sync::Arc<crate::indexes::data::DerivedIndexDefinition>,
            Option<std::sync::Arc<crate::indexes::data::DerivedIndexGeneration>>,
        )>,
        SelectedIndexGenerationAdmissionStop<Stop>,
    > {
        let state = self.state.read();
        let visits = navigation_work(state.definitions.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(visits, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(definition) = state.definitions.get(&index) else {
            return Ok(None);
        };
        let generation = state.generations.published_for_commit_admitted(
            index,
            definition.branch_scoped.then_some(branch),
            commit,
            version,
            &mut prepare,
        )?;
        Ok(Some((std::sync::Arc::clone(definition), generation)))
    }
    /// Check one selected published index under a single owner read lock.
    /// Each actual catalog descent and the branch-key copy is admitted before
    /// the corresponding read; this method never starts reconstruction.
    pub(crate) fn has_published_generation_for_observation_admitted<Stop>(
        &self,
        index: DerivedIndexId,
        observation: &RelationalBranchObservation,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, SelectedIndexGenerationAdmissionStop<Stop>> {
        let Some(commit) = observation.commit_id() else {
            return Ok(false);
        };
        let state = self.state.read();
        let definition_work = navigation_work(state.definitions.len(), 1)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        prepare(definition_work, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        let Some(definition) = state.definitions.get(&index) else {
            return Ok(false);
        };
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
