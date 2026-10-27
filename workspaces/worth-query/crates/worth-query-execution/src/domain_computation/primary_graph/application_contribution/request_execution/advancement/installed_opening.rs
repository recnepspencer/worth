//! Installed runtime owners lend one branded request for each public call.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn with_host_advancement<R>(
        &self,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        let world = std::sync::Arc::clone(&self.product_runtime.owner);
        let execution = QueryRequestExecution::open_control(
            world.execution_placement(),
            worth_execution::CancellationToken::new(),
            None,
        );
        execution.run_advancement(
            Some(world.owner_identity()),
            Some(
                self.product_runtime
                    .source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            body,
        )
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
        self.open_funded(
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
            worth_execution::ExecutionAllocationPolicy::Execution(_) => self.open_funded(
                allocation_policy,
                request.cancellation().execution_token(),
                request.deadline(),
                body,
            ),
        }
    }

    fn open_funded<R>(
        &self,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let _custody = OpeningCustody::enter()?;
        let world = std::sync::Arc::clone(&self.product_runtime.owner);
        let execution = match allocation_policy {
            worth_execution::ExecutionAllocationPolicy::SystemAllocation => {
                QueryRequestExecution::open_control(
                    world.execution_placement(),
                    cancellation,
                    deadline,
                )
            }
            worth_execution::ExecutionAllocationPolicy::Execution(lease) => {
                QueryRequestExecution::under_lease(lease, cancellation, deadline)
            }
        };
        execution.run_advancement(
            Some(world.owner_identity()),
            Some(
                self.product_runtime
                    .source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            body,
        )
    }

    /// Checks borrowed advancement custody before a Publication entry reader.
    pub fn validate_application_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryForeignAdvancementPhase> {
        phase
            .execution_request_for(&self.product_runtime)
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
        let _custody = OpeningCustody::enter()?;
        let world = std::sync::Arc::clone(&self.product_runtime.owner);
        let execution = QueryRequestExecution::open(world.execution_placement(), request);
        execution.run_advancement(
            Some(world.owner_identity()),
            Some(
                self.product_runtime
                    .source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
            ),
            body,
        )
    }
}

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

    fn open_host_advancement<R>(
        &self,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
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
