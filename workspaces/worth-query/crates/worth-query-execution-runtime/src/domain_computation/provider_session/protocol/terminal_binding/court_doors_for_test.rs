//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProviderSessionTerminalBinding {
    #[doc(hidden)]
    pub fn same_session_for_test(&self, other: &Self) -> bool {
        Self::same_session(self, other)
    }
}
