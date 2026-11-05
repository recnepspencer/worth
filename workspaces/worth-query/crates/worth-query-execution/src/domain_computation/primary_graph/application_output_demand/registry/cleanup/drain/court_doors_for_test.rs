//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn drain_terminal_cleanup_admitted_for_test(
        &self,
        owner: &SourceInvalidationOwner,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::drain_terminal_cleanup_admitted(self, owner)
    }
}
