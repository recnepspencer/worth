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

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn discard_custody_for_test(&mut self, commit: &Commit) {
        Self::discard_custody(self, commit)
    }
}

impl DemandRegistryState {
    #[doc(hidden)]
    pub fn note_custody_changes_for_test(
        &mut self,
        edit: impl FnMut(&Commit, &mut SourceCustody) -> bool,
    ) {
        Self::note_custody_changes(self, edit)
    }
}
