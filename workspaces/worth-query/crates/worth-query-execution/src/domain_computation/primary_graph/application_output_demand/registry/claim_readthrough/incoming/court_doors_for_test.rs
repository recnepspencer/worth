//! Sealed test doors over this module's private members.
use super::*;

impl ClaimBatch {
    #[doc(hidden)]
    pub fn deactivate_for_test(&self) {
        Self::deactivate(self)
    }
}
