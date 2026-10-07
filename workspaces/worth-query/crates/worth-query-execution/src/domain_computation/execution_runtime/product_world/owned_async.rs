use super::WorthQueryProductRuntime;

impl WorthQueryProductRuntime {
    pub fn admit_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        Result<
            worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
            worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
        >,
        super::WorthQueryHandleDenial,
    > {
        self.gate.with_runtime(|_| {
            self.owner.admit_owned_async_request(
                bridge,
                declaration,
                selected.observation(),
                selected.bridge_source_observation(),
            )
        })
    }

    pub fn revalidate_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        Result<
            worth_runtime_bridge::facade::BridgeOwnedAsyncRevalidationAdmission,
            worth_runtime_world::facade::RuntimeWorldOwnedAsyncRevalidationDenial,
        >,
        super::WorthQueryHandleDenial,
    > {
        self.gate.with_runtime(|_| {
            self.owner.revalidate_owned_async_request(
                bridge,
                request,
                selected.observation(),
                selected.bridge_source_observation(),
            )
        })
    }
}
