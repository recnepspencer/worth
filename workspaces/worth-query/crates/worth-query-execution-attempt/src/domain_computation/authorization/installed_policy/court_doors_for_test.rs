//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInstalledAuthorizationRegistry {
    #[doc(hidden)]
    pub fn bridge_for_test(&self) -> &BridgeAuthorizationRuntime {
        Self::bridge(self)
    }
}

impl WorthQueryInstalledAuthorizationRegistry {
    #[doc(hidden)]
    pub fn capability_plan_for_test<Schema, Capability, Operation, Input>(
        &self,
        capability: &worth_query_installation::facade::WorthQueryInstalledApplicationCapability<
            Schema,
            Capability,
            Operation,
            Input,
        >,
    ) -> Option<&WorthQueryInstalledCapabilityPlan> {
        Self::capability_plan::<Schema, Capability, Operation, Input>(self, capability)
    }
}

impl WorthQueryInstalledAuthorizationRegistry {
    #[doc(hidden)]
    pub fn elevation_lifecycle_operation_for_test<Schema, Operation>(
        &self,
        operation: &str,
        input_type: &str,
    ) -> Result<
        Option<(
            [u8; 32],
            [u8; 32],
            WorthQueryElevationLifecycleOperationRole,
        )>,
        (),
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
    {
        Self::elevation_lifecycle_operation::<Schema, Operation>(self, operation, input_type)
    }
}
