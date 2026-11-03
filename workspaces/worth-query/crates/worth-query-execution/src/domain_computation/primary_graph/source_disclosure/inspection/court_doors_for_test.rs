//! Sealed test doors over this module's private members.
use super::*;

impl OriginalSourceInspectionDisclosure {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &WorthQueryObservedScopeSelector {
        Self::scope(self)
    }
}
