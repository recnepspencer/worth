//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn new_for_test(key_identity: [u8; 32], intent_identity: [u8; 32]) -> Self {
        Self::new(key_identity, intent_identity)
    }
}

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn bind_governed_proposal_for_test(
        self,
        governed_proposal_identity: Option<&[u8; 32]>,
    ) -> Self {
        Self::bind_governed_proposal(self, governed_proposal_identity)
    }
}

impl WorthQueryApplicationIdempotencyBinding {
    #[doc(hidden)]
    pub fn key_text_for_test(self) -> String {
        Self::key_text(self)
    }
}
