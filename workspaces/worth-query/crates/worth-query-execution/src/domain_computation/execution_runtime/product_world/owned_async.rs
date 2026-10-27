use super::WorthQueryProductRuntime;

impl WorthQueryProductRuntime {
    pub(in crate::domain_computation) fn admit_managed_owned_async_request(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeOwnedAsyncManagedRequestAdmission,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.owner.admit_managed_owned_async_request(
            bridge,
            declaration,
            selected.observation(),
            selected.bridge_source_observation(),
        )
    }

    pub(in crate::domain_computation) fn advance_owned_async_source_clock(
        &self,
        bridge: &worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly,
        declaration: &worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
        selected: &crate::basis::WorthQueryProductBranchLease,
        coordinate: u64,
    ) -> Result<
        worth_signal::facade::TemporalClockAdvanceSummary,
        worth_runtime_world::facade::RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    > {
        self.owner.advance_owned_async_source_clock(
            bridge,
            declaration,
            selected.observation(),
            selected.bridge_source_observation(),
            coordinate,
        )
    }

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
}
