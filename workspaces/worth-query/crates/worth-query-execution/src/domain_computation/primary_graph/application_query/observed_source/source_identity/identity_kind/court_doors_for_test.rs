//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCheckpointSourceIdentity {
    #[doc(hidden)]
    pub fn new_for_test(identity: [u8; 32]) -> Self {
        Self::new(identity)
    }
}

impl WorthQueryRuntimeSourceIdentity {
    #[doc(hidden)]
    pub fn new_for_test(identity: [u8; 32]) -> Self {
        Self::new(identity)
    }
}

impl WorthQueryCheckpointSourceIdentity {
    #[doc(hidden)]
    pub fn bytes_for_test(self) -> [u8; 32] {
        Self::bytes(self)
    }
}
