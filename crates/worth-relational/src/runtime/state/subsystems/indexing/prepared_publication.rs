//! A funded catalog payload remains private until its native candidate performs.
use super::{generation_catalog::GenerationCatalog, IndexingState, IndexingSubsystem};
use crate::indexes::data::{DerivedIndexGeneration, SelectedIndexGenerationAdmissionStop};
use crate::runtime::state::subsystems::RuntimeOwnedState;

#[derive(Debug)]
pub(crate) struct PreparedGenerationBatch {
    catalog: RuntimeOwnedState<IndexingState>,
    generations: Vec<DerivedIndexGeneration>,
}
impl PreparedGenerationBatch {
    pub(crate) fn publish(self) {
        let mut state = self.catalog.write();
        for generation in self.generations {
            state.generations.publish(generation);
        }
    }
}
impl IndexingSubsystem {
    pub(crate) fn prepare_generation_batch<Stop>(
        &self,
        generations: Vec<DerivedIndexGeneration>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<PreparedGenerationBatch, SelectedIndexGenerationAdmissionStop<Stop>> {
        GenerationCatalog::prepare_deferred_batch(&generations, &mut prepare)?;
        // The generation payload and issued catalog binding are carried, not rebuilt.
        prepare(1, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(PreparedGenerationBatch {
            catalog: self.state.share(),
            generations,
        })
    }
}
