//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderIdempotencyLayout {
    #[doc(hidden)]
    pub fn key_locator_for_test(&self) -> &AspectFieldLocator {
        &self.key_locator
    }
}
