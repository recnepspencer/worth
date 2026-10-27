use super::WorthQueryProductRuntime;

impl WorthQueryProductRuntime {
    pub fn admit_unqueued_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.owner.admit_unqueued_owned_async_request(
            bridge,
            declaration,
            selected.observation(),
            selected.bridge_source_observation(),
        )
    }

    pub fn admit_unqueued_owned_async_request_at_branch_head(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.owner
            .admit_unqueued_owned_async_request_at_branch_head(
                bridge,
                declaration,
                selected.observation(),
                selected.bridge_source_observation(),
            )
    }

    pub fn revalidate_custodied_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        request: &mut worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRevalidation,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncManagedRevalidationDenial,
    > {
        self.owner.revalidate_managed_owned_async_request(
            bridge,
            request,
            selected.observation(),
            selected.bridge_source_observation(),
        )
    }

    pub fn admit_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.owner.admit_owned_async_request(
            bridge,
            declaration,
            selected.observation(),
            selected.bridge_source_observation(),
        )
    }

    pub fn revalidate_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        request: &worth_runtime_bridge::facade::BridgeOwnedAsyncRequestAdmission,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncRevalidationAdmission,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRevalidationDenial,
    > {
        self.owner.revalidate_owned_async_request(
            bridge,
            request,
            selected.observation(),
            selected.bridge_source_observation(),
        )
    }
}
