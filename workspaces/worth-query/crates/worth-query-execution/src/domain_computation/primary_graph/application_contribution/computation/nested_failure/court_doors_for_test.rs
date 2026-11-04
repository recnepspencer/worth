//! Sealed test doors over this module's private members.
use super::*;

impl<E> MemberComputeFailure<E> {
    #[doc(hidden)]
    pub fn into_failure_for_test(self) -> E {
        Self::into_failure(self)
    }
}

impl<'context> NestedFailureTransport<'context> {
    #[doc(hidden)]
    pub fn call_for_test<T, E>(
        &mut self,
        run: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, MemberComputeFailure<E>> {
        Self::call(self, run)
    }
}
