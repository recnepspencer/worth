//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure;
use crate::domain_computation::vocabulary::primary_graph::original::OriginalSourceDisclosure;

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
