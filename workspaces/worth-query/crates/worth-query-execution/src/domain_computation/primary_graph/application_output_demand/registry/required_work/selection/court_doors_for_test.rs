//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn next_required_work_for_test(
        &self,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        Self::next_required_work(self, source_owner)
    }
}
