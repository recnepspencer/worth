//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryGraphWorkManagedRunIdentity {
    #[doc(hidden)]
    pub fn mint_for_test() -> Option<Self> {
        Self::mint()
    }
}

impl WorthQueryGraphWorkSessionIdentity {
    #[doc(hidden)]
    pub fn mint_for_test() -> Option<Self> {
        Self::mint()
    }
}
