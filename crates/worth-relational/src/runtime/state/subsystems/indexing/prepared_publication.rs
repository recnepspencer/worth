//! A funded catalog payload remains private until its native candidate performs.
use super::{generation_catalog::GenerationCatalog, IndexingState, IndexingSubsystem};
use crate::indexes::data::{DerivedIndexGeneration, SelectedIndexGenerationAdmissionStop};
use crate::runtime::state::subsystems::RuntimeOwnedState;

#[derive(Debug)]
pub(crate) struct PreparedGenerationBatch {
    catalog: RuntimeOwnedState<IndexingState>,
    generations: Vec<DerivedIndexGeneration>,
    basis: Option<super::PreparedIndexBasisSelection>,
}
impl PreparedGenerationBatch {
    pub(crate) fn selection_state(&self) -> Option<std::sync::Arc<super::IndexSelectionState>> {
        self.basis
            .as_ref()
            .map(super::PreparedIndexBasisSelection::selection_state)
    }

    pub(crate) fn publish(mut self) {
        let mut state = self.catalog.write();
        for generation in self.generations {
            let id = generation.generation_id;
            state.generations.publish(generation);
            if self.basis.is_none() {
                state.record_basis_generation(id);
            }
            if let Some(basis) = &mut self.basis {
                basis.replace(
                    state
                        .generations
                        .generation(id)
                        .expect("published generation"),
                );
            }
        }
        if let Some(basis) = self.basis {
            state.bases.install(basis);
        }
    }
}
impl IndexingSubsystem {
    pub(crate) fn prepare_generation_batch<Stop>(
        &self,
        generations: Vec<DerivedIndexGeneration>,
        basis: Option<super::PreparedIndexBasisSelection>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<PreparedGenerationBatch, SelectedIndexGenerationAdmissionStop<Stop>> {
        if let Some(basis) = &basis {
            basis.quote_bindings(generations.len(), &mut prepare)?;
        } else {
            self.state
                .read()
                .bases
                .quote_updates(&generations, true, &mut prepare)?;
        }
        GenerationCatalog::prepare_deferred_batch(&generations, &mut prepare)?;
        // The generation payload and issued catalog binding are carried, not rebuilt.
        prepare(1, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(PreparedGenerationBatch {
            catalog: self.state.share(),
            generations,
            basis,
        })
    }
}
