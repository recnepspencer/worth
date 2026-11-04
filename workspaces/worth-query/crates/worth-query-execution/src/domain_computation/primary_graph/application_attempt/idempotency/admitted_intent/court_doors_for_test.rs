//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn bind_admission_for_test<S, O, I, E>(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<S, O, I, E>,
    ) -> Self {
        Self::bind_admission::<S, O, I, E>(self, admission)
    }
}
