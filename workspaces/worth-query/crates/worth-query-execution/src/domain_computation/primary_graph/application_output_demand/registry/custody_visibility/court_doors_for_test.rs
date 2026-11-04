//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn custody_prunable_for_test(&self, commit: &CompositeCommitIdentity) -> bool {
        Self::custody_prunable(self, commit)
    }
}
