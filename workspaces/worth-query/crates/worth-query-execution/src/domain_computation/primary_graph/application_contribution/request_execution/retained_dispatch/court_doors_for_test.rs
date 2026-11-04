//! Sealed test doors over this module's private members.
use super::*;

impl QueryRequestExecution<'_> {
    #[doc(hidden)]
    pub fn with_native_lease_for_test<R>(
        &self,
        use_lease: impl FnOnce(&ExecutionResourceLease<'_>) -> R,
    ) -> Result<R, worth_execution::WorkCeilingDenial> {
        Self::with_native_lease(self, use_lease)
    }
}
