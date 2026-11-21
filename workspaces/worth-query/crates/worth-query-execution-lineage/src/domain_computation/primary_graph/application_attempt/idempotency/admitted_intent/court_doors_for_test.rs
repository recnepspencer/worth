//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn bind_admission_for_test<A: AdmittedOperationPort + ?Sized>(self, admission: &A) -> Self {
        Self::bind_admission(self, admission)
    }
}
