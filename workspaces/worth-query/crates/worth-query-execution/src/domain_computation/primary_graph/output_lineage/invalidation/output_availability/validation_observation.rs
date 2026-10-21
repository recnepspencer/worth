//! Read-only native receipt restricted to the armed owning validation control.
use super::*;
use crate::domain_computation::primary_graph::application_runtime::WorthQuerySourcePublicationHeadForTest as Head;
use crate::domain_computation::primary_graph::application_runtime::WorthQuerySourcePublicationImageForTest as View;
impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn source_publication_validation_image_for_test(
        &self,
        observations: &[Arc<ObservedSourcePublication>],
        selected: &PositionedRelationalSnapshot,
    ) -> Result<View, CompanionPreflightStop> {
        let cell = self
            .cell_for_read(selected)?
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        let image = cell.read_image();

        let mut outer_publications = Vec::with_capacity(observations.len());
        for observed in observations {
            let state = image
                .payload()
                .current
                .source_publications
                .get(&observed.address)
                .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;

            let mut heads = Vec::with_capacity(state.heads.len());
            for head in state.heads.iter() {
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
