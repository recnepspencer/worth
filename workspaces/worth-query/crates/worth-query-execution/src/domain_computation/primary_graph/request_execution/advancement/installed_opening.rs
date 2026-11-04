//! An installed product runtime lends one branded request for each public call.
use super::*;

impl crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime {
    /// Runs one host call inside the installed World's request policy.
    /// Custody follows the owner into descendant workers; unrelated calls own theirs.
    /// Zero memory is `PolicyMemoryLimit` at serial placement and a policy
    /// `MemoryLimit` at leased placement, before the callback begins.
    pub fn with_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.open_host_advancement(worth_execution::CancellationToken::new(), None, body)
    }

    /// Runs a scoped host call under this installed World's policy and the
    /// caller's cancellation and deadline, lending only its branded phase.
    pub fn with_request_advancement<R>(
        &self,
        request: &WorthQueryRequestScope,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        self.open_host_advancement(
            request.cancellation().execution_token(),
            request.deadline(),
            body,
        )
    }

    /// Runs one host call funded as `allocation_policy` says, under `cancellation` and
    /// `deadline`. Under `Execution(lease)` the call's budget is a controlled child of
    /// `lease`; `SystemAllocation` uses the World's placement.
    pub(in crate::domain_computation::primary_graph) fn with_funded_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let lease = match allocation_policy {
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                return self.open_host_advancement(cancellation, deadline, body);
            }
            worth_execution::ExecutionAllocationPolicy::Execution(lease) => lease,
        };
        let _custody = OpeningCustody::enter()?;
        QueryRequestExecution::under_lease(lease, cancellation, deadline).run_advancement(
            Some(self.owner.owner_identity()),
            Some(
                self.source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            body,
        )
    }

    fn open_host_advancement<R>(
        &self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        QueryRequestExecution::open_control(
            self.owner.execution_placement(),
            cancellation,
            deadline,
        )
        .run_advancement(
            Some(self.owner.owner_identity()),
            Some(
                self.source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            body,
        )
    }
}
