//! Sealed test doors over this module's private members.
use super::*;

impl OriginalProjection {
    #[doc(hidden)]
    pub fn allocation_bytes_for_test() -> u64 {
        Self::allocation_bytes()
    }
}
