use worth_runtime_world::facade::{
    ProductBranchObservation, RuntimeWorldObservationPort, RuntimeWorldPublicationPort,
    RuntimeWorldRecoveryPort,
};

use std::sync::Arc;

use super::runtime::WorthQueryProductRootIdentity;

/// One admitted product occurrence carried into a provider attempt. The weak
/// ports cannot extend the root's admission lifetime or choose a different head.
#[derive(Clone)]
pub struct WorthQueryProductPublicationBinding {
    program_source: BranchProgramSourceCache,
    pub(super) adoption_preparation: Arc<super::adoption_preparation::AdoptionPreparationControl>,
    observation: ProductBranchObservation,
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    recovery: RuntimeWorldRecoveryPort,
    observation_port: RuntimeWorldObservationPort,
    clock: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldClock,
    root_identity: Arc<WorthQueryProductRootIdentity>,
}

impl WorthQueryProductPublicationBinding {
    pub(super) fn new(
        adoption_preparation: Arc<super::adoption_preparation::AdoptionPreparationControl>,
        observation: ProductBranchObservation,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        recovery: RuntimeWorldRecoveryPort,
        observation_port: RuntimeWorldObservationPort,
        clock: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldClock,
        root_identity: Arc<WorthQueryProductRootIdentity>,
        branch_program_owner: Option<Arc<worth_proof::InstalledBranchProgramSourceOwner>>,
    ) -> Self {
        Self {
            program_source: BranchProgramSourceCache::for_owner(branch_program_owner),
            adoption_preparation,
            observation,
            publication,
            observation_port,
            recovery,
            clock,
            root_identity,
        }
    }

    pub(crate) fn observation(&self) -> &ProductBranchObservation {
        &self.observation
    }

    pub(crate) fn publication(&self) -> &RuntimeWorldPublicationPort<(), (), (), (), ()> {
        &self.publication
    }

    pub(crate) fn recovery(&self) -> RuntimeWorldRecoveryPort {
        self.recovery.clone()
    }

    pub(in crate::domain_computation) fn observation_port(&self) -> &RuntimeWorldObservationPort {
        &self.observation_port
    }

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }

    pub(super) fn admits_clock(
        &self,
        clock: &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> bool {
        self.clock.admission_clock.same_clock(clock)
    }

    pub(crate) fn deadline(
        &self,
        deadline: worth_execution::MonotonicDeadline,
    ) -> Option<worth_runtime_world::facade::RuntimeWorldInstant> {
        self.clock.deadline(deadline)
    }
}

impl super::WorthQueryProductRuntime {
    pub(crate) fn publication_binding(
        &self,
        observation: ProductBranchObservation,
    ) -> WorthQueryProductPublicationBinding {
        WorthQueryProductPublicationBinding::new(
            Arc::clone(&self.adoption_preparation),
            observation,
            self.owner.publication_port(),
            self.owner.recovery_port(),
            self.owner.observation_port(),
            self.clock.clone(),
            self.root_identity(),
            self.branch_program_source_owner.clone(),
        )
    }
}

mod court_doors_for_test;
mod selected_program;
use selected_program::BranchProgramSourceCache;
