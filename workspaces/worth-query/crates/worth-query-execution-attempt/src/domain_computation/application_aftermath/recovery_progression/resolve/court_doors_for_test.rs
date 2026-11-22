//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryAdmittedIdempotencyRead {
    #[doc(hidden)]
    pub fn mint_for_test(
        binding: WorthQueryApplicationIdempotencyBinding,
        resolution: WorthQueryApplicationIdempotencyResolution,
    ) -> Self {
        Self::mint(binding, resolution)
    }
}
