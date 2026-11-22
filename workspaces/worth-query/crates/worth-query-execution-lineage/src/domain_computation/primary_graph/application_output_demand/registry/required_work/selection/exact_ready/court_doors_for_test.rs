//! Sealed test doors over this module's private members.
use super::*;

impl SelectedReadyReadmission {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        Self::key(self)
    }
}
