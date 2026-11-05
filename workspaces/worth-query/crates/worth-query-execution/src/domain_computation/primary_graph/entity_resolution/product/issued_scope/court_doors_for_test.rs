//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Scope> WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope> {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &WorthQueryApplicationEntityIdentity<Schema, Scope> {
        Self::scope(self)
    }
}
