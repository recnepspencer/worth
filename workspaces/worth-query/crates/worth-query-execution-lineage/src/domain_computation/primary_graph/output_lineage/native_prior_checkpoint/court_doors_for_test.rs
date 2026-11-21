//! Sealed test doors over this module's private members.
use super::*;

impl NativePriorCheckpointLocator {
    #[doc(hidden)]
    pub fn producer_for_test(&self) -> &str {
        Self::producer(self)
    }
}

impl NativePriorCheckpointLocator {
    #[doc(hidden)]
    pub fn source_for_test(&self) -> [u8; 32] {
        Self::source(self)
    }
}
