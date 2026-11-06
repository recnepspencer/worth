//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn bind_producing_implementation_for_test(
        self,
        implementation: RequiredOutputProducingImplementation,
    ) -> Self {
        Self::bind_producing_implementation(self, implementation)
    }
}
