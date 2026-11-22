//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryInstalledApplicationBridge {
    #[doc(hidden)]
    pub fn ordinary_for_test(&self) -> &RuntimeBridge {
        Self::ordinary(self)
    }
}
