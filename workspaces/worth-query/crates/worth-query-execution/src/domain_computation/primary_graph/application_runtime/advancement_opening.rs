//! Installed runtime owners lend one branded request for each public call, through the
//! product runtime they install.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::request_execution::advancement::WorthQueryAdvancementSource as _;
use {crate::domain_computation::request_execution::advancement::WorthQueryAdvancementDenial, crate::domain_computation::request_execution::advancement::WorthQueryAdvancementPhase, crate::domain_computation::request_execution::advancement::WorthQueryForeignAdvancementPhase};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn with_host_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.runtime.product_runtime.with_advancement(body)
    }

    /// Runs one host call funded as `allocation_policy` says. Under
    /// `Execution(lease)` the call's budget is a controlled child of `lease`,
    /// so every worker descends from it and the caller's charges stay inside
    /// one budget; any other lease is unrelated to those workers and refuses
    /// `UnrelatedNestedLease`. `SystemAllocation` uses the World's placement.
    pub fn with_funded_host_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.runtime.product_runtime.with_funded_advancement(
            allocation_policy,
            worth_execution::CancellationToken::new(),
            None,
            body,
        )
    }

    /// [`Self::with_application_advancement`] funded as `allocation_policy`
    /// says, under the request's cancellation and deadline. A lane that takes
    /// the caller's allocation policy opens through this, so `Execution(lease)`
    /// is the call's one budget rather than a second one inside the World's.
    pub fn with_funded_application_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        match allocation_policy {
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                self.with_application_advancement(request, body)
            }
            worth_execution::ExecutionAllocationPolicy::Execution(_) => {
                self.runtime.product_runtime.with_funded_advancement(
                    allocation_policy,
                    request.cancellation().execution_token(),
                    request.deadline(),
                    body,
                )
            }
        }
    }

    /// Checks borrowed advancement custody before a Publication entry reader.
    pub fn validate_application_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryForeignAdvancementPhase> {
        phase
            .execution_request_for(&self.runtime.product_runtime)
            .map(|_| ())
    }

    /// The higher-ranked callback prevents a phase from escaping in its result.
    /// Custody covers the host call and its descendant workers; unrelated calls own theirs.
    /// Zero memory reports `PolicyMemoryLimit` in serial placement and a policy
    /// `MemoryLimit` in leased placement; both refuse before any read.
    pub fn with_application_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.runtime
            .product_runtime
            .with_request_advancement(request, body)
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
