//! An installed product runtime lends one branded request for each public call.
use super::WorthQueryProductRuntime;
use crate::domain_computation::request_execution::advancement::source::WorthQueryAdvancementSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use {
    crate::domain_computation::request_execution::advancement::installed_opening::open_leased_advancement,
    crate::domain_computation::request_execution::advancement::installed_opening::open_placed_advancement,
};
use {
    crate::domain_computation::request_execution::advancement::WorthQueryAdvancementDenial,
    crate::domain_computation::request_execution::advancement::WorthQueryAdvancementPhase,
};

impl WorthQueryProductRuntime {
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
    pub(in crate::domain_computation) fn with_funded_advancement<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        match allocation_policy {
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                self.open_host_advancement(cancellation, deadline, body)
            }
            worth_execution::ExecutionAllocationPolicy::Execution(lease) => {
                open_leased_advancement(lease, self, cancellation, deadline, body)
            }
        }
    }

    fn open_host_advancement<R>(
        &self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<worth_execution::MonotonicDeadline>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        open_placed_advancement(
            self.owner.execution_placement(),
            self,
            cancellation,
            deadline,
            body,
        )
    }
}

impl WorthQueryAdvancementSource for WorthQueryProductRuntime {
    fn advancement_owner(&self) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity> {
        Some(self.owner.owner_identity())
    }
    fn advancement_source_instance(&self) -> u64 {
        self.source
            .authoritative_source_profile()
            .runtime_instance_id()
    }
}
