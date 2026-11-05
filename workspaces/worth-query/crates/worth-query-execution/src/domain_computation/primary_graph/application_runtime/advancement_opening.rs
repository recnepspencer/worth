//! The runtime's consumer advancement doors: each opens through the execution runtime's
//! application advancement.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::execution_runtime::ApplicationAdvancementOps;
use crate::domain_computation::request_execution::advancement::WorthQueryAdvancementSource as _;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use {
    crate::domain_computation::request_execution::advancement::WorthQueryAdvancementDenial,
    crate::domain_computation::request_execution::advancement::WorthQueryAdvancementPhase,
    crate::domain_computation::request_execution::advancement::WorthQueryForeignAdvancementPhase,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn with_host_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        <Self as ApplicationAdvancementOps<Schema>>::with_host_advancement(self, body)
    }

    /// Runs one host call funded as `allocation_policy` says; see the execution runtime's
    /// application advancement.
    pub fn with_funded_host_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        <Self as ApplicationAdvancementOps<Schema>>::with_funded_host_advancement(
            self,
            allocation_policy,
            body,
        )
    }

    /// [`Self::with_application_advancement`] funded as `allocation_policy` says, under the
    /// request's cancellation and deadline.
    pub fn with_funded_application_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        <Self as ApplicationAdvancementOps<Schema>>::with_funded_application_advancement(
            self,
            request,
            allocation_policy,
            body,
        )
    }

    /// Checks borrowed advancement custody before a Publication entry reader.
    pub fn validate_application_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryForeignAdvancementPhase> {
        <Self as ApplicationAdvancementOps<Schema>>::validate_application_advancement(self, phase)
    }

    /// The higher-ranked callback prevents a phase from escaping in its result.
    pub fn with_application_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        <Self as ApplicationAdvancementOps<Schema>>::with_application_advancement(
            self, request, body,
        )
    }
}

impl<Schema: ApplicationSchema>
    crate::domain_computation::request_execution::advancement::WorthQueryAdvancementSource
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn advancement_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity> {
        self.runtime.product_runtime.advancement_owner()
    }
    fn advancement_source_instance(&self) -> u64 {
        self.runtime.product_runtime.advancement_source_instance()
    }
}
