//! Sealed test doors over this module's private members.
use super::*;

impl<T> RetainedFact<T> {
    #[doc(hidden)]
    pub fn ordinal_for_test(&self) -> &usize {
        &self.ordinal
    }
}

impl<T> RetainedFact<T> {
    #[doc(hidden)]
    pub fn value_for_test(&self) -> &T {
        &self.value
    }
}

impl StoreDenial {
    #[doc(hidden)]
    pub fn into_attempt_denial_for_test(
        self,
        subject: &str,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial {
        Self::into_attempt_denial(self, subject)
    }
}

impl<T> RetainedFact<T> {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &AdmittedFactKey {
        &self.key
    }
}
