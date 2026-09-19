use super::builder::{MissingRuntimeWorldInput, RuntimeWorldOwnerBuilder};
use super::service_ports::{
    RuntimeWorldBranchPort, RuntimeWorldLifecyclePort, RuntimeWorldObservationPort,
    RuntimeWorldPublicationPort, RuntimeWorldRecoveryPort,
};
use super::{RuntimeWorldOwnerInputs, RuntimeWorldOwnerRoot};
use crate::identity::{RuntimeWorldIdentityExhaustion, RuntimeWorldOwnerIdentity};
use std::sync::Arc;

/// Sole strong World owner. Ports borrow its lifetime through weak references.
/// Dropping it stops admission; already admitted custody remains accountable.
pub struct RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) root: Arc<RuntimeWorldOwnerRoot<D, I, E, Ctx, T>>,
}
impl RuntimeWorldOwner<(), (), (), (), ()> {
    pub fn builder() -> RuntimeWorldOwnerBuilder<
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
        MissingRuntimeWorldInput,
    > {
        RuntimeWorldOwnerBuilder::new()
    }
}
impl<D, I, E, Ctx, T> RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn admit_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        observation: &crate::branch::ProductBranchObservation,
        relational_source: &worth_relational::facade::bridge::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        super::owner::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.root
            .admit_owned_async_request(bridge, declaration, observation, relational_source)
    }

    pub fn revalidate_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        observation: &crate::branch::ProductBranchObservation,
        relational_source: &worth_relational::facade::bridge::RelationalBridgeObservationLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRevalidationAdmission,
        super::owner::RuntimeWorldOwnedAsyncRevalidationDenial,
    > {
        self.root
            .revalidate_owned_async_request(bridge, request, observation, relational_source)
    }

    pub(super) fn from_inputs(
        inputs: RuntimeWorldOwnerInputs<D, I, E, Ctx, T>,
    ) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        Ok(Self {
            root: Arc::new(RuntimeWorldOwnerRoot::new(inputs)?),
        })
    }
    pub fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.root.owner_identity()
    }
    pub fn publication_port(&self) -> RuntimeWorldPublicationPort<D, I, E, Ctx, T> {
        RuntimeWorldPublicationPort::new(Arc::downgrade(&self.root))
    }
    pub fn observation_port(&self) -> RuntimeWorldObservationPort {
        let service: Arc<dyn super::ports::RuntimeWorldObservationService + Send + Sync> =
            self.root.clone();
        RuntimeWorldObservationPort::new(Arc::downgrade(&service))
    }
    pub fn branch_port(&self) -> RuntimeWorldBranchPort {
        let service: Arc<dyn super::ports::RuntimeWorldBranchService + Send + Sync> =
            self.root.clone();
        RuntimeWorldBranchPort::new(Arc::downgrade(&service))
    }
    pub fn lifecycle_port(&self) -> RuntimeWorldLifecyclePort {
        let service: Arc<dyn super::ports::RuntimeWorldLifecycleService + Send + Sync> =
            self.root.clone();
        RuntimeWorldLifecyclePort::new(Arc::downgrade(&service))
    }
    #[cfg(feature = "test-operation-control")]
    pub fn operation_control(&self) -> super::RuntimeWorldOperationControl {
        self.root.state.operation_control.clone()
    }

    pub fn inspection_port(&self) -> super::RuntimeWorldInspectionPort {
        let service: Arc<dyn crate::inspection::RuntimeWorldInspectionService + Send + Sync> =
            self.root.clone();
        super::RuntimeWorldInspectionPort::new(Arc::downgrade(&service))
    }
    pub fn recovery_port(&self) -> RuntimeWorldRecoveryPort {
        let service: Arc<dyn super::ports::RuntimeWorldRecoveryService + Send + Sync> =
            self.root.clone();
        RuntimeWorldRecoveryPort::new(Arc::downgrade(&service))
    }
}
impl<D, I, E, Ctx, T> Drop for RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn drop(&mut self) {
        self.root.release_public_owner();
    }
}
