//! Read-only native receipt restricted to the armed owning validation control.
use super::*;
use crate::domain_computation::primary_graph::application_runtime::{
    WorthQuerySourcePublicationHeadForTest as Head, WorthQuerySourcePublicationImageForTest as View,
};
impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn source_publication_validation_image_for_test(
        &self,
        observations: &[&Arc<ObservedSourcePublication>],
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<View, CompanionPreflightStop> {
        let cell = self
            .cell_for_read(selected, admission)?
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let image = cell.read_image();
        let outer_bytes = observations
            .len()
            .checked_mul(std::mem::size_of::<Vec<Head>>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.admit_read_scratch(outer_bytes)?;
        let mut outer_publications = Vec::with_capacity(observations.len());
        for observed in observations {
            admission.ordered_read(image.payload().current.source_publications.len())?;
            let state = image
                .payload()
                .current
                .source_publications
                .get(&observed.address)
                .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
            let head_bytes = state
                .heads
                .len()
                .checked_mul(std::mem::size_of::<Head>())
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            admission.admit_read_scratch(head_bytes)?;
            let mut heads = Vec::with_capacity(state.heads.len());
            for head in state.heads.iter() {
                let width = u64::try_from(head.key.source.output_binding.as_str().len())
                    .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
                admission.admit_read_scratch(width)?;
                admission.charge_external_work(1 + width)?;
                let (occurrence, generation, slot) = head.identity.address();
                heads.push(Head {
                    binding: head.key.source.output_binding.as_str().to_owned(),
                    partition: head.key.partition,
                    occurrence,
                    generation,
                    slot,
                });
            }
            outer_publications.push(heads);
        }
        Ok(View {
            root_id: image.root_id(),
            native_commit: image.commit_id().map(|id| id.0),
            native_position: image.position().map(|position| position.0),
            topology_generation: image.topology_generation(),
            outer_publications,
        })
    }
}
