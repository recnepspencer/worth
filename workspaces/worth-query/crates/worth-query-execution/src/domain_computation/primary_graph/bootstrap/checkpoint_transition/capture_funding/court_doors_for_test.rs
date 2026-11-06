//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCheckpointTransitionResources {
    #[doc(hidden)]
    pub fn capture_lease_for_test<'scope, 'authority>(
        self,
        request: ExecutionRequest<'scope, 'authority>,
        policy: ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Result<Option<ExecutionResourceLease<'authority>>, WorkCeilingDenial> {
        Self::capture_lease(self, request, policy)
    }
}
