//! The Home's private World adapter. Only World decodes owner payloads.
mod attachment;
mod close;
mod definition;
pub use close::{WorldHomeClosePosture, WorldHomeCloseRefused, WorldSealedApplicationHome};
pub use definition::{ApplicationHomeWorldDefinition, ApplicationWorldDefinition};
mod ports;
mod publication;
mod publication_fate;
mod publication_task;
pub use ports::ApplicationWorldPorts;
pub(crate) use publication_task::WorldPublications;
mod query_root;
mod readmission;
pub use query_root::ApplicationHomeQueryRootDenial;
mod recovery;
pub(crate) use attachment::ApplicationHomeBridgeComposition;
pub(crate) use attachment::UnclaimedWorldSlot;
pub use readmission::{readmit_query_world, ApplicationHomeQueryCompositionDenial};
pub use readmission::{readmit_world, WorldHomeReadmissionRefused, WorldReadmittedApplicationHome};
use std::sync::Arc;
use worth_runtime_world::facade::WorldDurableHomeBinding;
use worth_store::physical_runtime::{AdmittedRecordPlacementPolicy, ServingPhysicalRuntime};
pub(crate) struct ApplicationWorldMedium {
    serving: Arc<ServingPhysicalRuntime>,
    binding: WorldDurableHomeBinding,
    placement: AdmittedRecordPlacementPolicy,
    publications: Arc<WorldPublications>,
}
impl std::fmt::Debug for ApplicationWorldMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationWorldMedium")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}
impl worth_runtime_world::facade::RuntimeWorldDurableMedium for ApplicationWorldMedium {
    fn binding(&self) -> &WorldDurableHomeBinding {
        &self.binding
    }
    fn publication_task_posture(&self) -> worth_runtime_world::facade::WorldPublicationTaskPosture {
        self.publications.posture()
    }
    fn recover(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_runtime_world::facade::WorldDurableRecoverySource,
        worth_runtime_world::facade::WorldDurableMediumDenial,
    > {
        recovery::read(self, execution)
    }
    fn advance_publication(
        &self,
        subject: &worth_runtime_world::facade::WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> worth_runtime_world::facade::WorldPublicationObservation {
        publication::publish(self, subject, execution)
    }
}
