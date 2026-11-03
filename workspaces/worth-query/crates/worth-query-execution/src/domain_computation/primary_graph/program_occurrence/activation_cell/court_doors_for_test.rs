//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProgramActivationCell {
    #[doc(hidden)]
    pub fn unpublished_for_test() -> Self {
        Self::unpublished()
    }
}
