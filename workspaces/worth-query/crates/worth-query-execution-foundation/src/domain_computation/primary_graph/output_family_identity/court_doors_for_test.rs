//! Sealed test doors over this module's private members.
use super::*;

impl OutputFamilyIdentity {
    #[doc(hidden)]
    pub fn declared_for_test(identity: &'static str) -> Self {
        Self::declared(identity)
    }
}

impl OutputFamilyIdentity {
    #[doc(hidden)]
    pub fn as_str_for_test(&self) -> &str {
        Self::as_str(self)
    }
}
