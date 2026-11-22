//! Sealed test doors over this module's private members.
use super::*;

impl Denial {
    #[doc(hidden)]
    pub fn pending_reason_for_test(self) -> Pending {
        Self::pending_reason(self)
    }
}
