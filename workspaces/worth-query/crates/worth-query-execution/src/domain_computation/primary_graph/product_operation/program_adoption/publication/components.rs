use worth_runtime_world::facade::{NoEffectCompositePublication, RuntimeWorldPublicationOutcome};

use super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::domain_computation::primary_graph::product_operation::program_adoption::preparation::WorthQueryPreparedBranchAdoption;

pub enum WorthQueryBranchAdoptionPublicationOutcome {
    Performed(WorthQueryPerformedBranchAdoption),
    NoEffect(NoEffectCompositePublication),
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
            activation_registry,
            ..
        } = self;
        match publication.execute() {
            RuntimeWorldPublicationOutcome::Performed(performed) => {
                activation_registry.record_program_publication(
                    performed.new_product_head().branch_identity(),
                    performed.new_product_head().lifecycle_incarnation(),
                    &target,
                );
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
