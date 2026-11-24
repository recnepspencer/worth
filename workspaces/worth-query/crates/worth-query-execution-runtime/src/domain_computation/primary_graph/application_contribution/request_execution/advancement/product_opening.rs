//! Product runtime entry points lending branded advancement custody.
use super::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

impl crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime {
    /// Runs one host call inside the installed World's request policy.
    /// Custody follows the owner into descendant workers; unrelated calls own theirs.
    /// Zero memory is `PolicyMemoryLimit` at serial placement and a policy
    /// `MemoryLimit` at leased placement, before the callback begins.
    pub fn with_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            self.with_advancement_on_execution_stack(body)
        })
    }

    #[inline(never)]
    fn with_advancement_on_execution_stack<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.open_host_advancement(worth_execution::CancellationToken::new(), None, false, body)
    }

    /// Runs a scoped host call under this installed World's policy and the
    /// caller's cancellation and deadline, lending only its branded phase.
    pub fn with_request_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            self.with_request_advancement_on_execution_stack(request, false, body)
        })
    }

    /// Runs a scoped call with explicit retention of the returned outcome.
    pub(crate) fn with_request_advancement_retaining<R>(
        &self,
        request: &WorthQueryRequestScope,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        worth_execution::run_with_execution_stack(move || {
            let _custody = OpeningCustody::enter()?;
            self.with_execution_placement(|placement| {
                let execution = QueryRequestExecution::open(placement, request);
                execution.run_product_advancement(self, retain_returned_outcome, body)
            })?
        })
    }

    /// Runs a call funded by the caller's explicit allocation policy.
    pub(crate) fn with_funded_product_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        self.with_execution_placement(|placement| {
            let execution = match allocation_policy {
                worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                    QueryRequestExecution::open_control(placement, cancellation, deadline)
                }
                worth_execution::ExecutionAllocationPolicy::Execution(lease) => {
                    QueryRequestExecution::under_lease(lease, cancellation, deadline)
                }
            };
            execution.run_product_advancement(self, retain_returned_outcome, body)
        })?
    }

    /// Runs a call carrying the caller's exact original `ExecutionRequest`
    /// (both leased and lease-free `ExecutionRequest::serial`), preserving its
    /// work ceiling, memory limit, deadline, and cancellation end-to-end.
    pub(crate) fn with_execution_request_advancement<R>(
        &self,
        caller: worth_execution::ExecutionRequest<'_, '_>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        self.with_execution_placement(|placement| {
            let execution = match caller.allocation_policy() {
                worth_execution::ExecutionAllocationPolicy::Execution(lease) => {
                    QueryRequestExecution::under_lease(
                        lease,
                        worth_execution::CancellationToken::new(),
                        caller.deadline(),
                    )
                }
                worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                    QueryRequestExecution::open_serial_request(placement, caller)
                }
            };
            execution.run_product_advancement(self, false, body)
        })?
    }

    #[inline(never)]
    fn with_request_advancement_on_execution_stack<R>(
        &self,
        request: &WorthQueryRequestScope,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.open_host_advancement(
            request.cancellation().execution_token(),
            request.deadline(),
            retain_returned_outcome,
            body,
        )
    }

    fn open_host_advancement<R>(
        &self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        retain_returned_outcome: bool,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        self.with_execution_placement(|placement| {
            QueryRequestExecution::open_control(placement, cancellation, deadline)
                .run_product_advancement(self, retain_returned_outcome, body)
        })?
    }
}
