//! Sealed test doors over this module's private members.
use super::*;

impl<'member> WorthQueryMemberExecution<'member> {
    #[doc(hidden)]
    pub fn compute_for_test<T, E>(
        self,
        run: impl FnOnce(
            WorthQueryManagedComputationExecution<'member>,
            &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
            &mut NestedFailureTransport<'member>,
        ) -> Result<T, E>,
    ) -> Result<T, E> {
        Self::compute(self, run)
    }
}

impl<'member> WorthQueryMemberExecution<'member> {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        WorthQueryManagedComputationExecution<'member>,
        &'member mut dyn FnMut() -> Result<(), worth_execution::MapKernelStop>,
    ) {
        Self::into_parts(self)
    }
}
