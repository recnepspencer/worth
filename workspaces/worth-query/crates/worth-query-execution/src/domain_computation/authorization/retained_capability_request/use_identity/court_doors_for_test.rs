//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCapabilityUseIdentity {
    #[doc(hidden)]
    pub fn retain_request_for_test(
        request: &WorthQueryRetainedCapabilityRequest,
        execution: &ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Self, Cause> {
        Self::retain_request(request, execution, allocation)
    }
}
