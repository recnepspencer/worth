use worth_runtime_world::facade::{NoEffectCompositePublication, RuntimeWorldPublicationOutcome};

use super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::domain_computation::primary_graph::product_operation::program_adoption::preparation::WorthQueryPreparedBranchAdoption;

/// What publishing a prepared branch adoption did. Returned by `publish`.
pub enum WorthQueryBranchAdoptionPublicationOutcome {
    /// The publication call could not enter its host-owned execution request.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The adoption is published and the product head moved.
    Performed(WorthQueryPerformedBranchAdoption),
    /// The publication had no effect: no owner and no product reference moved.
    NoEffect(NoEffectCompositePublication),
    /// Some owners moved, but the product head did not. Recover publication.
    ProductUnpublished(WorthQueryUnpublishedBranchAdoption),
}

impl WorthQueryPreparedBranchAdoption {
    pub fn publish(self) -> WorthQueryBranchAdoptionPublicationOutcome {
        let Some(owner) = self.owner.upgrade() else {
            let request = self.host_request.clone();
            return crate::domain_computation::primary_graph::application_contribution::with_serial_host_advancement(
                self.owner_identity, self.host_policy, &request, |phase| self.publish_in_advancement(&phase),
            ).unwrap_or_else(WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied);
        };
        let request = self.host_request.clone();
        crate::domain_computation::primary_graph::application_contribution::with_world_advancement(
            &owner,
            &request,
            |phase| self.publish_in_advancement(&phase),
        )
        .unwrap_or_else(WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied)
    }

    fn publish_in_advancement(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorthQueryBranchAdoptionPublicationOutcome {
        let execution = phase
            .request_for_owner(self.owner_identity)
            .expect("the adoption call opened on its retained owner identity");
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
        match publication.execute(execution) {
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
