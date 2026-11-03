//! Sealed test doors over this module's private members.
use super::*;

impl InstalledProducerEdition {
    #[doc(hidden)]
    pub fn for_test_for_test(digest: [u8; 32]) -> Self {
        Self::for_test(digest)
    }
}
