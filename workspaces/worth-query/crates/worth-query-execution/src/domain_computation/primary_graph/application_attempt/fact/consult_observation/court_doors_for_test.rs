//! Sealed test doors over this module's private members.
use super::*;

impl ReadConsultObservationForTest {
    #[doc(hidden)]
    pub fn consults_for_test(&self) -> usize {
        Self::consults(self)
    }
}
