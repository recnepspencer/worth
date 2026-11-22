//! Sealed test doors over this module's private members.
use super::*;

impl OriginalSourceInspectionDisclosure {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &WorthQueryObservedScopeSelector {
        Self::scope(self)
    }
}

impl OriginalSourceInspectionDisclosure {
    #[doc(hidden)]
    pub fn association_for_test(&self) -> &InstalledSourceDisclosureBinding {
        Self::association(self)
    }
}

impl OriginalSourceInspectionDisclosure {
    #[doc(hidden)]
    pub fn disclosure_for_test(&self) -> &OriginalSourceDisclosure {
        Self::disclosure(self)
    }
}
