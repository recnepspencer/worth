//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn new_for_test(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
        Self::new(key_identity, intent_identity)
    }
}
