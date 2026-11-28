//! Installed runtime owners lend one branded request for each public call.
use super::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase, WorthQueryBootstrapAdvancementPhase,
    WorthQueryForeignAdvancementPhase,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
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
        worth_execution::run_with_execution_stack(move || {
            self.runtime
                .product_runtime
                .with_funded_product_advancement(
                    allocation_policy,
                    worth_execution::CancellationToken::new(),
                    None,
                    false,
                    body,
                )
        })
    }

    /// Runs one host call carrying the caller's exact original
    /// [`worth_execution::ExecutionRequest`] (leased or lease-free serial),
    /// preserving its work ceiling, memory limit, deadline, and cancellation.
    pub fn with_execution_request_advancement<R>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            self.runtime
                .product_runtime
                .with_execution_request_advancement(execution, body)
        })
    }

    /// Runs one host call carrying the caller's exact original
    /// [`worth_execution::ExecutionRequest`], narrowed by BOTH the installed
    /// World placement and the session's policy and request scope.
    pub fn with_scoped_execution_request_advancement<R>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        policy: worth_foundational::ExecutionRequestPolicy,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            self.runtime
                .product_runtime
                .with_scoped_execution_request_advancement(execution, policy, request, body)
        })
    }

    /// Hands off an active bootstrap opening phase into a Home advancement
    /// on this installed World while preserving the opening request's controls.
    pub fn with_bootstrap_handed_off_advancement<R>(
        &self,
        bootstrap: &WorthQueryBootstrapAdvancementPhase<'_>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.runtime
            .product_runtime
            .with_bootstrap_handed_off_advancement(bootstrap, retain_returned_outcome, body)
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
        worth_execution::run_with_execution_stack(move || {
            self.with_funded_application_advancement_on_execution_stack(
                request,
                allocation_policy,
                false,
                body,
            )
        })
    }

    /// Runs an effectful application call under the original request and funding.
    /// Once its callback returns, transfers that exact owned outcome even if
    /// nested work stopped. This is not a commit certificate and does not clear
    /// cancellation or admit any further effects. Pre-entry refusal still wins.
    pub fn with_funded_application_completion<R>(
        &self,
        request: &WorthQueryRequestScope,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            self.with_funded_application_advancement_on_execution_stack(
                request,
                allocation_policy,
                true,
                body,
            )
        })
    }

    #[inline(never)]
    fn with_funded_application_advancement_on_execution_stack<R>(
        &self,
        request: &WorthQueryRequestScope,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        match allocation_policy {
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => self
                .runtime
                .product_runtime
                .with_request_advancement_retaining(request, retain_returned_outcome, body),
            worth_execution::ExecutionAllocationPolicy::Execution(_) => self
                .runtime
                .product_runtime
                .with_funded_product_advancement(
                    allocation_policy,
                    request.cancellation().execution_token(),
                    request.deadline(),
                    retain_returned_outcome,
                    body,
                ),
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
            .with_request_advancement_retaining(request, false, body)
    }
}
