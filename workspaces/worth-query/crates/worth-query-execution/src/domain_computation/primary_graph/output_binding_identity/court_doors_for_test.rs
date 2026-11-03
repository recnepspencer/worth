//! Sealed test doors over this module's private members.
use super::*;

impl OutputBindingIdentity {
    #[doc(hidden)]
    pub fn declared_for_test(name: &str) -> Self {
        Self::declared(name)
    }
}
