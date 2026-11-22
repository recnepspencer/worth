//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCommittedAftermathCausality {
    #[doc(hidden)]
    pub fn owned_receipt_backing_bytes_for_test(&self) -> Option<u64> {
        Self::owned_receipt_backing_bytes(self)
    }
}
