//! Sealed test doors over this module's private members.
use super::*;

impl TouchAdmissionFailure {
    #[doc(hidden)]
    pub fn into_invariant_failure_for_test(
        self,
    ) -> crate::domain_computation::WorthQueryInvariantExecutionFailure {
        Self::into_invariant_failure(self)
    }
}
