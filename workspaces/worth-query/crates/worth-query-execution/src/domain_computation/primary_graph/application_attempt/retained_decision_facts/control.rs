use super::StoreDenial;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Attempt-local borrowing only; this facet is never retained in a snapshot,
/// prepared candidate, completed read set or commit authority.
#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct StorageControl<'scope, 'authority> {
    policy: ExecutionAllocationPolicy<'scope, 'authority>,
    request: Option<&'scope WorthQueryRequestScope>,
}
impl<'scope, 'authority> StorageControl<'scope, 'authority> {
    /// None is explicit only when the projection has no admitted Query request.
    /// Admitted projection MUST pass Some(admission.publication_request()).
    pub(in crate::domain_computation::primary_graph) fn new(
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
        request: Option<&'scope WorthQueryRequestScope>,
    ) -> Self {
        Self { policy, request }
    }
    pub(in crate::domain_computation::primary_graph) fn policy(
        self,
    ) -> ExecutionAllocationPolicy<'scope, 'authority> {
        self.policy
    }
    pub(in crate::domain_computation::primary_graph) fn request(
        self,
    ) -> Option<
        &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    > {
        self.request
    }
    pub(in crate::domain_computation::primary_graph) fn check_live(
        self,
    ) -> Result<(), StoreDenial> {
        self.policy.check_live()?;
        if let Some(interruption) = self.request.and_then(WorthQueryRequestScope::interruption) {
            return Err(StoreDenial::RequestInterruption(interruption));
        }
        Ok(())
    }
}
