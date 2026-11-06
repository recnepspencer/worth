//! Sealed test doors over this module's private members.
use super::*;

impl RebaseVerificationFailure {
    #[doc(hidden)]
    pub fn into_reason_for_test(self) -> Reason {
        Self::into_reason(self)
    }
}

impl RebaseVerificationFailure {
    #[doc(hidden)]
    pub fn reason_for_test(&self) -> &Reason {
        Self::reason(self)
    }
}
