//! Sealed test doors over this module's private members.
use super::*;

impl ProjectionDisposal {
    #[doc(hidden)]
    pub fn admitted_for_test(
        execution: &ExecutionRequest<'_, '_>,
        allocation: &SourceFactsAllocation,
        maximum_depth: usize,
    ) -> Result<Self, Cause> {
        Self::admitted(execution, allocation, maximum_depth)
    }
}

impl OwnedProjectionRows {
    #[doc(hidden)]
    pub fn managed_for_test(
        rows: Vec<WorthQueryApplicationProjectionNode>,
        disposal: ProjectionDisposal,
        payload_bytes: usize,
        allocation: Arc<SourceFactsAllocation>,
    ) -> Self {
        Self::managed(rows, disposal, payload_bytes, allocation)
    }
}
