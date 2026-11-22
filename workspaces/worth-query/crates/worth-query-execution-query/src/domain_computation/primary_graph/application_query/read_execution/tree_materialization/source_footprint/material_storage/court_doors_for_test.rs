//! Sealed test doors over this module's private members.
use super::*;

impl<'a, 'request, 'authority> MaterialFootprintStorage<'a, 'request, 'authority> {
    #[doc(hidden)]
    pub fn new_for_test(
        execution: &'a ExecutionRequest<'request, 'authority>,
        allocation: Option<&'a Arc<SourceFactsAllocation>>,
    ) -> Self {
        Self::new(execution, allocation)
    }
}

impl<'a, 'request, 'authority> MaterialFootprintStorage<'a, 'request, 'authority> {
    #[doc(hidden)]
    pub fn normalize_for_test(
        &self,
        footprint: &mut WorthQueryObservedSourceFootprint,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        Self::normalize(self, footprint)
    }
}
