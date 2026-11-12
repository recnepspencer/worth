//! Existing owned-async owner operations through the same weak publication port.
use super::*;
use crate::lifecycle::{
    RuntimeWorldOwnedAsyncManagedRevalidationDenial as RevalidationDenial,
    RuntimeWorldOwnedAsyncRequestAdmissionDenial as AdmissionDenial,
};
use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncManagedRequestAdmission, BridgeOwnedAsyncManagedRevalidation,
    BridgeSealedRuntimeAssembly, LoweredBridgeAsyncSourceDeclaration,
    RelationalBridgeObservationLease,
};

impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn admit_managed_owned_async_request(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        source: &RelationalBridgeObservationLease,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, AdmissionDenial> {
        self.service()
            .map_err(|_| AdmissionDenial::OwnerUnavailable)?
            .admit_managed_owned_async_request(bridge, declaration, observation, source)
    }

    pub fn advance_owned_async_source_clock(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        source: &RelationalBridgeObservationLease,
        coordinate: u64,
    ) -> Result<worth_signal::facade::TemporalClockAdvanceSummary, AdmissionDenial> {
        self.service()
            .map_err(|_| AdmissionDenial::OwnerUnavailable)?
            .advance_owned_async_source_clock(bridge, declaration, observation, source, coordinate)
    }

    pub fn admit_unqueued_owned_async_request(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        source: &RelationalBridgeObservationLease,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, AdmissionDenial> {
        self.service()
            .map_err(|_| AdmissionDenial::OwnerUnavailable)?
            .admit_unqueued_owned_async_request(bridge, declaration, observation, source)
    }

    pub fn admit_unqueued_owned_async_request_at_branch_head(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        observation: &ProductBranchObservation,
        source: &RelationalBridgeObservationLease,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, AdmissionDenial> {
        self.service()
            .map_err(|_| AdmissionDenial::OwnerUnavailable)?
            .admit_unqueued_owned_async_request_at_branch_head(
                bridge,
                declaration,
                observation,
                source,
            )
    }

    pub fn revalidate_managed_owned_async_request(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        request: &mut BridgeOwnedAsyncManagedRequestAdmission,
        observation: &ProductBranchObservation,
        source: &RelationalBridgeObservationLease,
    ) -> Result<BridgeOwnedAsyncManagedRevalidation, RevalidationDenial> {
        self.service()
            .map_err(|_| RevalidationDenial::OwnerUnavailable)?
            .revalidate_managed_owned_async_request(bridge, request, observation, source)
    }
}
