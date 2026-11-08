//! A funded catalog payload remains private until its native candidate performs.
use super::generation_publication_gate::{GenerationPublicationGate, GenerationPublicationPermit};
use super::{IndexingState, IndexingSubsystem};
use crate::history::data::BranchId;
use crate::indexes::data::{DerivedIndexGeneration, SelectedIndexGenerationAdmissionStop};
use crate::runtime::state::subsystems::RuntimeOwnedState;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct PreparedGenerationBatch {
    // Abandonment releases coordination before destroying owning payloads.
    stage: CatalogPublicationStage,
    catalog: RuntimeOwnedState<IndexingState>,
    generations: Vec<DerivedIndexGeneration>,
    branch_keys: Vec<Option<BranchId>>,
    gate: Arc<GenerationPublicationGate>,
}

#[derive(Debug)]
enum CatalogPublicationStage {
    Unadmitted,
    Admitted(GenerationPublicationPermit),
    Published,
}
impl PreparedGenerationBatch {
    pub(crate) fn covers_home_definitions(
        &self,
        definitions: &super::PreparedRelationalIndexDefinitions,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, worth_execution::WorkCeilingDenial> {
        let (covered, _) = execution.run_native_with_checkpoint(|context| {
            for index in definitions.introduced_ids() {
                let mut found = false;
                for generation in &self.generations {
                    if context.has_accounting() {
                        context.checkpoint(1)?;
                    } else {
                        context.poll()?;
                    }
                    if generation.index_id == index {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Ok(false);
                }
            }
            Ok::<_, worth_execution::MapKernelStop>(true)
        })?;
        covered.map_err(worth_execution::WorkCeilingDenial::Stopped)
    }

    pub(crate) fn admit_actual_geometry<Stop>(
        &mut self,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), SelectedIndexGenerationAdmissionStop<Stop>> {
        assert!(
            matches!(self.stage, CatalogPublicationStage::Unadmitted),
            "one preflight admits one candidate's publication"
        );
        let permit = self
            .gate
            .enter(|| prepare(0, 0))
            .map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        // Do not invoke payment callbacks under the indexing state lock.
        let quote = self
            .catalog
            .read()
            .generations
            .deferred_quote(&self.generations)
            .ok_or(SelectedIndexGenerationAdmissionStop::AccountingOverflow)?;
        quote.admit(&mut prepare)?;
        let mut keys = Vec::with_capacity(self.generations.len());
        for generation in &self.generations {
            if let Err(stop) = prepare(0, 0) {
                drop(permit);
                return Err(SelectedIndexGenerationAdmissionStop::Admission(stop));
            }
            keys.push(Some(generation.applicability.branch_id.clone()));
        }
        self.branch_keys = keys;
        self.stage = CatalogPublicationStage::Admitted(permit);
        Ok(())
    }

    /// Fixed installation retains Vec backing and unused lookup keys for
    /// owning cleanup after the caller's borrowed handoff has ended.
    pub(crate) fn publish_admitted(&mut self) {
        let mut state = self.catalog.write();
        let CatalogPublicationStage::Admitted(permit) =
            std::mem::replace(&mut self.stage, CatalogPublicationStage::Published)
        else {
            unreachable!("only the consuming cutover installs an admitted batch once")
        };
        assert_eq!(
            self.generations.len(),
            self.branch_keys.len(),
            "admission retains one exact lookup key per generation"
        );
        for (generation, key) in self.generations.drain(..).zip(self.branch_keys.iter_mut()) {
            state.generations.publish_prepared(generation, key);
        }
        drop(state);
        // No global gate survives an after-linearization pause or settlement.
        drop(permit);
    }

    pub(crate) fn is_published(&self) -> bool {
        matches!(self.stage, CatalogPublicationStage::Published)
    }
}
impl IndexingSubsystem {
    pub(crate) fn prepare_generation_batch<Stop>(
        &self,
        generations: Vec<DerivedIndexGeneration>,
        mut prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<PreparedGenerationBatch, SelectedIndexGenerationAdmissionStop<Stop>> {
        // Payload construction is paid here. Actual metadata geometry is admitted
        // by preflight just before lending the borrowed cutover, not speculated.
        prepare(2, 0).map_err(SelectedIndexGenerationAdmissionStop::Admission)?;
        Ok(PreparedGenerationBatch {
            catalog: self.state.share(),
            generations,
            branch_keys: Vec::new(),
            gate: Arc::clone(&self.generation_publication),
            stage: CatalogPublicationStage::Unadmitted,
        })
    }
}

#[cfg(test)]
mod tests;
