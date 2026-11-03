//! Sealed test doors over this module's private members.
use super::*;

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn note_custody_for_test(&mut self, commit: &Commit) {
        Self::note_custody(self, commit)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn pending_custody_prune_for_test(&self) -> bool {
        Self::pending_custody_prune(self)
    }
}
