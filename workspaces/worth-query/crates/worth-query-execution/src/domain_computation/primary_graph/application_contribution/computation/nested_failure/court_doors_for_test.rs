//! Sealed test doors over this module's private members.
use super::*;

impl<E> MemberComputeFailure<E> {
    #[doc(hidden)]
    pub fn into_failure_for_test(self) -> E {
        Self::into_failure(self)
    }
}
