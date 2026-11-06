//! Sealed test doors over this module's private members.
use super::*;

impl PriorAbsence {
    #[doc(hidden)]
    pub fn full_cause_for_test(&self) -> Cause {
        Self::full_cause(self)
    }
}
