use super::StoreDenial;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Attempt-local borrowing only; this facet is never retained in a snapshot,
/// prepared candidate, completed read set or commit authority.
#[derive(Clone, Copy)]
pub(in crate::domain_computation) struct StorageControl<'scope, 'authority> {
    execution: Option<worth_execution::ExecutionRequest<'scope, 'authority>>,
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
    request: Option<&'scope WorthQueryRequestScope>,
}
impl<'scope, 'authority> StorageControl<'scope, 'authority> {
    /// None is explicit only when the projection has no admitted Query request.
    /// Admitted projection MUST pass Some(admission.publication_request()).
    pub(in crate::domain_computation) fn new(
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
        request: Option<&'scope WorthQueryRequestScope>,
    ) -> Self {
        Self {
            policy,
            request,
            execution: None,
        }
    }
    pub(in crate::domain_computation) fn with_execution(
        mut self,
        execution: worth_execution::ExecutionRequest<'scope, 'authority>,
    ) -> Self {
        self.execution = Some(execution);
        self
    }
    pub(in crate::domain_computation) fn execution(
        self,
    ) -> Option<worth_execution::ExecutionRequest<'scope, 'authority>> {
        self.execution
    }
    pub fn policy(self) -> ExecutionAllocationPolicy<'scope, 'authority> {
        self.policy
    }
    pub(in crate::domain_computation) fn request(
        self,
    ) -> Option<
        &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    > {
        self.request
    }
    pub(in crate::domain_computation) fn check_live(self) -> Result<(), StoreDenial> {
        if let Some(execution) = self.execution {
            execution.check_live().map_err(|cause| {
                StoreDenial::ExecutionRequest(worth_execution::WorkCeilingDenial::Stopped(cause))
            })?;
        }
        self.policy.check_live()?;
        if let Some(interruption) = self.request.and_then(WorthQueryRequestScope::interruption) {
            return Err(StoreDenial::RequestInterruption(interruption));
        }
        Ok(())
    }
}
