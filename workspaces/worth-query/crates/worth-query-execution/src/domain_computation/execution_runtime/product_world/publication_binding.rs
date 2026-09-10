use worth_runtime_world::facade::{
    ProductBranchObservation, RuntimeWorldPublicationPort, RuntimeWorldRecoveryPort,
};

use std::sync::Arc;

use super::runtime::WorthQueryProductRootIdentity;

/// One admitted product occurrence carried into a provider attempt. The weak
/// ports cannot extend the root's admission lifetime or choose a different head.
#[derive(Clone)]
pub(crate) struct WorthQueryProductPublicationBinding {
    observation: ProductBranchObservation,
    publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
    recovery: RuntimeWorldRecoveryPort,
    clock: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldClock,
    root_identity: Arc<WorthQueryProductRootIdentity>,
}

impl WorthQueryProductPublicationBinding {
    pub(crate) fn new(
        observation: ProductBranchObservation,
        publication: RuntimeWorldPublicationPort<(), (), (), (), ()>,
        recovery: RuntimeWorldRecoveryPort,
        clock: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldClock,
        root_identity: Arc<WorthQueryProductRootIdentity>,
    ) -> Self {
        Self {
            observation,
            publication,
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

    pub(crate) fn root_identity(&self) -> Arc<WorthQueryProductRootIdentity> {
        Arc::clone(&self.root_identity)
    }

    pub(crate) fn deadline(
        &self,
        deadline: std::time::Instant,
    ) -> worth_runtime_world::facade::RuntimeWorldInstant {
        self.clock.deadline(deadline)
    }
}
