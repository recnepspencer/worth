//! Sealed test doors over this module's private members.
use super::*;

impl ComputationRegistration {
    #[doc(hidden)]
    pub fn retained_for_test() -> Self {
        Self::retained()
    }
}
