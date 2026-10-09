//! Installed runtime owners lend one branded request for each public call.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn with_host_advancement<R>(
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

    /// The higher-ranked callback prevents a phase from escaping in its result.
    /// Custody is per thread: another thread's public call owns its own request.
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
    /// Custody is per thread; another thread's public call owns its own request.
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
            Some(request.deadline()),
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
