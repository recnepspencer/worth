use worth_runtime_world::facade::{NoEffectCompositePublication, RuntimeWorldPublicationOutcome};

use super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::domain_computation::primary_graph::product_operation::program_adoption::preparation::WorthQueryPreparedBranchAdoption;

/// What publishing a prepared branch adoption did. Returned by `publish`.
pub enum WorthQueryBranchAdoptionPublicationOutcome {
    /// The adoption is published and the product head moved.
    Performed(WorthQueryPerformedBranchAdoption),
    /// The publication had no effect: no owner and no product reference moved.
    NoEffect(NoEffectCompositePublication),
    /// Some owners moved, but the product head did not. Recover publication.
    ProductUnpublished(WorthQueryUnpublishedBranchAdoption),
}

impl WorthQueryPreparedBranchAdoption {
    pub fn publish(self) -> WorthQueryBranchAdoptionPublicationOutcome {
        let Self {
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            publication,
            recovery,
            disposition,
            support_custody,
            ..
        } = self;
        match publication.execute() {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                WorthQueryBranchAdoptionPublicationOutcome::Performed(
                    WorthQueryPerformedBranchAdoption::new(
                        performed.consume(),
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                    ),
                )
            }
            RuntimeWorldPublicationOutcome::NoEffect(no_effect) => {
                WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect)
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(unpublished) => {
                WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(
                    WorthQueryUnpublishedBranchAdoption::new(
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                        unpublished,
                        recovery,
                        disposition,
                        support_custody,
                    ),
                )
            }
        }
    }
}
