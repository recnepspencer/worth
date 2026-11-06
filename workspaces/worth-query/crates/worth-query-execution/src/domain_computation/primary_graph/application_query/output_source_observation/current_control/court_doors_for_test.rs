//! Sealed test doors over this module's private members.
use super::*;

impl SourceQueryObservationStop {
    #[doc(hidden)]
    pub fn is_control_refusal_for_test(&self) -> bool {
        Self::is_control_refusal(self)
    }
}
